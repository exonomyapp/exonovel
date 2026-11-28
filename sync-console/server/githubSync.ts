import crypto from 'node:crypto';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import {
  getSyncBaseSnapshot,
  loadSyncBaseIndex,
  removeSyncBaseSnapshot,
  runThreeWayMergeText,
  saveSyncBaseSnapshot,
} from './syncMerge.ts';

const require = createRequire(import.meta.url);
const ignoreFactory = require('ignore') as (options?: {
  ignorecase?: boolean;
  allowRelativePaths?: boolean;
}) => {
  add(patterns: string | readonly string[]): unknown;
  ignores(pathname: string): boolean;
  test(pathname: string): {
    ignored: boolean;
    unignored: boolean;
    rule?: { pattern: string; negative: boolean };
  };
};

const ROOT_DIR = process.cwd();
const GITIGNORE_PATH = path.join(ROOT_DIR, '.gitignore');
const PENDING_COMMIT_PATH = path.join(ROOT_DIR, '.local', 'pending-commit.json');
const PRODUCTION_SITE_URL = 'https://exonovel.exosystems.net';

export const GITHUB_OWNER = 'exonomyapp';
export const GITHUB_REPO = 'exonovel';
export const GITHUB_BASE_BRANCH = 'main';
const GITHUB_API_BASE = 'https://api.github.com';
const GITHUB_API_VERSION = '2022-11-28';

// Permanent structural/secret boundaries that must never be walked, un-ignored, or exposed.
const HARD_PROTECTED_DIRS = new Set([
  '.git',
  'node_modules',
  'dist',
  'build',
  'coverage',
  'target',
  '.aistudio',
  '.agents',
  '.local',
  '.neon',
  '.vite',
]);

const HARD_PROTECTED_BASENAMES = new Set(['.env', '.env.local', '.DS_Store']);

// Files generated/maintained strictly outside AI Studio (e.g. Rust CI / Dependabot)
// that must never be deleted or pushed from this workspace.
const REMOTE_MANAGED_HIDDEN_FILES = new Set(['Cargo.lock']);

// Core .gitignore safety patterns that cannot be removed through the UI.
const LOCKED_GITIGNORE_PATTERNS = new Set([
  'node_modules/',
  'node_modules',
  'build/',
  'dist/',
  'coverage/',
  'target/',
  '.DS_Store',
  '*.log',
  '.env*',
  '!.env.example',
  '.neon',
  '.local/',
]);

const CONSOLE_ROOT_FILES = new Set([
  '.env.example',
  '.gitignore',
  'AGENTS.md',
  'bun.lock',
  'index.html',
  'metadata.json',
  'package.json',
  'server.ts',
  'skills-lock.json',
  'tsconfig.json',
  'vite.config.ts',
  'workspace.md',
]);

const SUPERSEDED_EXACT_PATHS = new Set([
  'firebase-applet-config.json',
  'neon.ts',
  'public/assets/exonovel-google-logo.png',
]);

const BINARY_EXTENSIONS = new Set([
  '.png',
  '.jpg',
  '.jpeg',
  '.webp',
  '.gif',
  '.ico',
  '.bmp',
  '.woff',
  '.woff2',
  '.ttf',
  '.otf',
  '.wasm',
  '.zip',
  '.tar',
  '.gz',
  '.pdf',
]);

export type SyncFileCategory = 'app' | 'console' | 'superseded';

export interface WorkspaceDiffEntry {
  path: string;
  status: 'modified' | 'added_local' | 'remote_only' | 'deleted_remote';
  category: SyncFileCategory;
  sizeBytes: number;
  remoteSizeBytes?: number;
  remoteSha?: string;
  hasBase: boolean;
  remoteUpdatedSinceBase: boolean;
  canPush: boolean;
  canPull: boolean;
  canDeleteRemote: boolean;
}

export interface GitignoreRuleEntry {
  pattern: string;
  locked: boolean;
  matchedCount: number;
}

export interface IgnoredFileEntry {
  path: string;
  matchedRule: string;
  ruleLocked: boolean;
  existsLocal: boolean;
  existsRemote: boolean;
  sizeBytes: number;
  remoteSha?: string;
}

export interface PreparedCommitSuggestion {
  commitMessage: string;
  branchName: string;
  prTitle: string;
  prBody: string;
  source: 'authored' | 'fallback' | 'none';
}

export interface WorkspaceDiffResult {
  headSha: string;
  files: WorkspaceDiffEntry[];
  gitignore: {
    rules: GitignoreRuleEntry[];
    ignoredFiles: IgnoredFileEntry[];
  };
  commitSuggestion: PreparedCommitSuggestion;
}

export interface DiffHunkLine {
  type: 'ctx' | 'add' | 'del';
  oldLine: number | null;
  newLine: number | null;
  text: string;
}

export interface WorkspaceFileDiffDetail {
  path: string;
  existsLocal: boolean;
  existsRemote: boolean;
  remoteSha: string | null;
  hasBase: boolean;
  baseSha: string | null;
  remoteUpdatedSinceBase: boolean;
  isBinary: boolean;
  localSize: number;
  remoteSize: number;
  localContent: string | null;
  remoteContent: string | null;
  baseContent: string | null;
  diffHunks: DiffHunkLine[];
  diffTruncated: boolean;
  truncated: boolean;
}

export interface MergeConflictEntry {
  path: string;
  direction: 'pull' | 'push';
  reason: 'overlap' | 'remote_ahead_clean' | 'no_base' | 'binary' | 'deleted_remote';
  remoteSha: string;
  baseSha: string | null;
  isBinary: boolean;
  baseContent: string | null;
  localContent: string | null;
  remoteContent: string | null;
  suggestedContent: string | null;
}

export interface ProductionProbeItem {
  path: string;
  status: number;
  ok: boolean;
  note: string;
  latencyMs: number;
}

export interface ProductionProbeResult {
  siteUrl: string;
  checkedAt: string;
  allHealthy: boolean;
  results: ProductionProbeItem[];
}

export class GitHubSyncConflictError extends Error {
  conflicts: MergeConflictEntry[];
  headSha: string | null;

  constructor(
    message: string,
    options?: { conflicts?: MergeConflictEntry[]; headSha?: string | null }
  ) {
    super(message);
    this.name = 'GitHubSyncConflictError';
    this.conflicts = options?.conflicts || [];
    this.headSha = options?.headSha || null;
  }
}

function normalizeRelativePath(relPath: string): string {
  return relPath.replace(/\\/g, '/').replace(/^\/+/, '');
}

function isHardProtectedPath(relPath: string): boolean {
  const normalized = normalizeRelativePath(relPath);
  if (!normalized || normalized.includes('..') || normalized.includes('\0')) {
    return true;
  }
  const parts = normalized.split('/');
  for (const part of parts) {
    if (!part || part === '.' || part === '..') return true;
    if (HARD_PROTECTED_DIRS.has(part)) return true;
  }
  const base = parts[parts.length - 1];
  if (HARD_PROTECTED_BASENAMES.has(base)) return true;
  if (base.startsWith('.env.') && base !== '.env.example') return true;
  if (base.endsWith('.log')) return true;
  return false;
}

function isHiddenManagedPath(relPath: string): boolean {
  return REMOTE_MANAGED_HIDDEN_FILES.has(normalizeRelativePath(relPath));
}

function isLocalUploadSkipped(relPath: string): boolean {
  const normalized = normalizeRelativePath(relPath);
  return normalized.startsWith('public/assets/aistudio/');
}

function classifyPath(relPath: string): SyncFileCategory {
  const normalized = normalizeRelativePath(relPath);
  if (
    SUPERSEDED_EXACT_PATHS.has(normalized) ||
    normalized.startsWith('public/') ||
    normalized.startsWith('server/') ||
    normalized.startsWith('src/')
  ) {
    return 'superseded';
  }
  if (
    CONSOLE_ROOT_FILES.has(normalized) ||
    normalized.startsWith('sync-console/')
  ) {
    return 'console';
  }
  return 'app';
}

function resolveSafeSyncPath(
  relPath: string
): { relPath: string; absPath: string } | null {
  if (typeof relPath !== 'string') return null;
  const trimmed = relPath.trim().replace(/\\/g, '/');
  if (
    !trimmed ||
    trimmed.startsWith('/') ||
    isHardProtectedPath(trimmed) ||
    isHiddenManagedPath(trimmed)
  ) {
    return null;
  }
  const normalized = normalizeRelativePath(trimmed);
  const resolved = path.resolve(ROOT_DIR, normalized);
  if (!resolved.startsWith(ROOT_DIR + path.sep)) {
    return null;
  }
  return { relPath: normalized, absPath: resolved };
}

function readGitignoreLines(): { rawLines: string[]; rulePatterns: string[] } {
  if (!fs.existsSync(GITIGNORE_PATH)) {
    return { rawLines: [], rulePatterns: [] };
  }
  const content = fs.readFileSync(GITIGNORE_PATH, 'utf8');
  const rawLines = content.split(/\r?\n/);
  const rulePatterns: string[] = [];
  for (const line of rawLines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    rulePatterns.push(trimmed);
  }
  return { rawLines, rulePatterns };
}

function buildGitignoreMatcher() {
  const { rulePatterns } = readGitignoreLines();
  const matcher = ignoreFactory();
  if (rulePatterns.length > 0) {
    matcher.add(rulePatterns);
  }
  return { rulePatterns, matcher };
}

function sanitizeGitignorePattern(rawPattern: string): string {
  if (typeof rawPattern !== 'string') {
    throw new Error('Pattern must be a non-empty string.');
  }
  const cleaned = rawPattern.trim().replace(/\\/g, '/');
  if (
    !cleaned ||
    cleaned.length > 200 ||
    cleaned.includes('\n') ||
    cleaned.includes('\r') ||
    cleaned.includes('\0') ||
    cleaned.includes('..') ||
    cleaned.startsWith('#')
  ) {
    throw new Error('Invalid .gitignore pattern.');
  }
  return cleaned;
}

export function addGitignorePattern(rawPattern: string): { pattern: string } {
  const pattern = sanitizeGitignorePattern(rawPattern);
  const { rawLines, rulePatterns } = readGitignoreLines();

  if (rulePatterns.includes(pattern)) {
    return { pattern };
  }

  const nextLines = [...rawLines];
  while (nextLines.length > 0 && nextLines[nextLines.length - 1] === '') {
    nextLines.pop();
  }
  nextLines.push(pattern, '');
  fs.writeFileSync(GITIGNORE_PATH, nextLines.join('\n'), 'utf8');
  return { pattern };
}

export function removeGitignorePattern(rawPattern: string): { pattern: string } {
  const pattern = sanitizeGitignorePattern(rawPattern);
  if (LOCKED_GITIGNORE_PATTERNS.has(pattern)) {
    throw new Error(`Core safety rule "${pattern}" cannot be removed.`);
  }
  if (!fs.existsSync(GITIGNORE_PATH)) {
    throw new Error('.gitignore file does not exist.');
  }

  const { rawLines } = readGitignoreLines();
  let removed = false;
  const filtered: string[] = [];
  for (const line of rawLines) {
    if (!removed && line.trim() === pattern) {
      removed = true;
      continue;
    }
    filtered.push(line);
  }

  if (!removed) {
    throw new Error(`Rule "${pattern}" was not found in .gitignore.`);
  }

  while (filtered.length > 0 && filtered[filtered.length - 1] === '') {
    filtered.pop();
  }
  const output = filtered.length > 0 ? `${filtered.join('\n')}\n` : '';
  fs.writeFileSync(GITIGNORE_PATH, output, 'utf8');
  return { pattern };
}

export function unignoreFileInGitignore(rawPath: string): {
  action: 'removed_rule' | 'added_negation';
  pattern: string;
} {
  const resolved = resolveSafeSyncPath(rawPath);
  if (!resolved) {
    throw new Error(`Protected path cannot be un-ignored: ${String(rawPath)}`);
  }
  const { matcher, rulePatterns } = buildGitignoreMatcher();
  const testResult = matcher.test(resolved.relPath);
  if (!testResult.ignored) {
    return { action: 'removed_rule', pattern: resolved.relPath };
  }

  const matchedRule = testResult.rule?.pattern || '';
  if (LOCKED_GITIGNORE_PATTERNS.has(matchedRule)) {
    throw new Error(
      `"${resolved.relPath}" is ignored by protected safety rule "${matchedRule}".`
    );
  }

  const exactCandidates = new Set([resolved.relPath, `/${resolved.relPath}`]);
  if (
    matchedRule &&
    exactCandidates.has(matchedRule) &&
    rulePatterns.includes(matchedRule)
  ) {
    removeGitignorePattern(matchedRule);
    return { action: 'removed_rule', pattern: matchedRule };
  }

  const negation = `!/${resolved.relPath}`;
  addGitignorePattern(negation);
  return { action: 'added_negation', pattern: negation };
}

export function getGitHubToken(): string {
  return (process.env.GITHUB_PAT || '').trim();
}

async function githubRequest<T>(
  endpoint: string,
  options: {
    method?: string;
    body?: unknown;
    token: string;
  }
): Promise<{ data: T; headers: Headers }> {
  const url = `${GITHUB_API_BASE}${endpoint}`;
  const res = await fetch(url, {
    method: options.method || 'GET',
    headers: {
      Authorization: `Bearer ${options.token}`,
      Accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': GITHUB_API_VERSION,
      'User-Agent': 'ExoNovel-Workspace-Sync',
      ...(options.body !== undefined ? { 'Content-Type': 'application/json' } : {}),
    },
    ...(options.body !== undefined ? { body: JSON.stringify(options.body) } : {}),
  });

  const text = await res.text();
  let parsed: unknown = null;
  if (text) {
    try {
      parsed = JSON.parse(text);
    } catch {
      parsed = { message: text.slice(0, 300) };
    }
  }

  if (!res.ok) {
    const msg =
      parsed &&
      typeof parsed === 'object' &&
      'message' in parsed &&
      typeof (parsed as { message?: unknown }).message === 'string'
        ? (parsed as { message: string }).message
        : `GitHub API returned status ${res.status}`;
    throw new Error(`${msg} (HTTP ${res.status})`);
  }

  return { data: parsed as T, headers: res.headers };
}

function computeGitBlobSha(buffer: Buffer): string {
  const header = Buffer.from(`blob ${buffer.length}\0`, 'utf8');
  return crypto.createHash('sha1').update(header).update(buffer).digest('hex');
}

function collectWorkspaceFiles(dir: string, prefix = ''): string[] {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  const files: string[] = [];

  for (const entry of entries) {
    if (entry.isSymbolicLink()) continue;
    const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (
      isHardProtectedPath(rel) ||
      isHiddenManagedPath(rel) ||
      isLocalUploadSkipped(rel)
    ) {
      continue;
    }
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectWorkspaceFiles(fullPath, rel));
    } else if (entry.isFile()) {
      files.push(rel);
    }
  }
  return files;
}

async function runWithConcurrency<T, R>(
  items: T[],
  limit: number,
  worker: (item: T) => Promise<R>
): Promise<R[]> {
  const results: R[] = new Array(items.length);
  let nextIndex = 0;

  async function next(): Promise<void> {
    while (nextIndex < items.length) {
      const idx = nextIndex++;
      results[idx] = await worker(items[idx]);
    }
  }

  const workers = Array.from({ length: Math.min(limit, items.length) }, () => next());
  await Promise.all(workers);
  return results;
}

function loadPendingCommitMeta(): {
  commitMessage?: string;
  branchName?: string;
  prTitle?: string;
  prBody?: string;
} | null {
  try {
    if (!fs.existsSync(PENDING_COMMIT_PATH)) return null;
    const raw = JSON.parse(fs.readFileSync(PENDING_COMMIT_PATH, 'utf8')) as {
      commitMessage?: unknown;
      branchName?: unknown;
      prTitle?: unknown;
      prBody?: unknown;
    };
    if (typeof raw?.commitMessage === 'string' && raw.commitMessage.trim()) {
      return {
        commitMessage: raw.commitMessage.trim(),
        branchName:
          typeof raw.branchName === 'string' ? raw.branchName.trim() : undefined,
        prTitle: typeof raw.prTitle === 'string' ? raw.prTitle.trim() : undefined,
        prBody: typeof raw.prBody === 'string' ? raw.prBody.trim() : undefined,
      };
    }
  } catch {
    // Ignore malformed pending commit file
  }
  return null;
}

function clearPendingCommitMeta(): void {
  try {
    if (fs.existsSync(PENDING_COMMIT_PATH)) {
      fs.unlinkSync(PENDING_COMMIT_PATH);
    }
  } catch {
    // Ignore cleanup errors
  }
}

function buildPreparedCommitSuggestion(
  files: WorkspaceDiffEntry[]
): PreparedCommitSuggestion {
  const authored = loadPendingCommitMeta();
  if (authored?.commitMessage) {
    return {
      commitMessage: authored.commitMessage,
      branchName:
        authored.branchName ||
        `sync/exonovel-${new Date().toISOString().slice(0, 10)}`,
      prTitle: authored.prTitle || authored.commitMessage.split('\n')[0],
      prBody: authored.prBody || '',
      source: 'authored',
    };
  }

  const pushables = files.filter((f) => f.canPush);
  if (pushables.length === 0) {
    return {
      commitMessage: '',
      branchName: '',
      prTitle: '',
      prBody: '',
      source: 'none',
    };
  }

  const scopes = new Set<string>();
  for (const f of pushables) {
    if (f.path.startsWith('sync-console/')) scopes.add('sync-console');
    else if (f.path.startsWith('.github/')) scopes.add('ci');
    else if (f.path.startsWith('crates/')) scopes.add('app');
    else if (f.path.startsWith('style/')) scopes.add('style');
    else if (f.path.startsWith('docs/')) scopes.add('docs');
    else scopes.add('workspace');
  }

  const scopeLabel = Array.from(scopes).slice(0, 2).join(',');
  const shortNames = pushables
    .slice(0, 3)
    .map((f) => path.posix.basename(f.path))
    .join(', ');
  const more = pushables.length > 3 ? ` (+${pushables.length - 3} more)` : '';
  const verb = pushables.every((f) => f.path.startsWith('docs/')) ? 'docs' : 'chore';

  return {
    commitMessage: `${verb}(${scopeLabel}): update ${shortNames}${more}`,
    branchName: `sync/${Array.from(scopes)[0] || 'update'}`,
    prTitle: '',
    prBody: '',
    source: 'fallback',
  };
}

export async function probeProductionSite(): Promise<ProductionProbeResult> {
  const paths = ['/', '/api/health'];
  const results: ProductionProbeItem[] = [];

  for (const p of paths) {
    const start = Date.now();
    try {
      const probe = await fetch(`${PRODUCTION_SITE_URL}${p}`, {
        signal: AbortSignal.timeout(6000),
      });
      const latencyMs = Date.now() - start;
      let note = `${probe.status} OK`;
      let ok = probe.ok;

      if (p === '/api/health') {
        if (probe.ok) {
          const body = (await probe.json().catch(() => null)) as {
            runtime?: string;
          } | null;
          note = body?.runtime ? `${probe.status} (${body.runtime})` : `${probe.status}`;
        } else {
          note = `${probe.status} Error`;
        }
      } else if (p === '/') {
        const html = probe.ok ? await probe.text().catch(() => '') : '';
        if (probe.status === 502) {
          note = '502 Bad Gateway';
          ok = false;
        } else if (!probe.ok) {
          note = `HTTP ${probe.status}`;
          ok = false;
        } else if (!html.includes('ExoNovel')) {
          note = `${probe.status} (Unexpected body)`;
          ok = false;
        } else {
          note = `${probe.status} (ExoNovel HTML)`;
        }
      }

      results.push({ path: p, status: probe.status, ok, note, latencyMs });
    } catch (err) {
      results.push({
        path: p,
        status: 0,
        ok: false,
        note: err instanceof Error ? err.message : 'Network error',
        latencyMs: Date.now() - start,
      });
    }
  }

  return {
    siteUrl: PRODUCTION_SITE_URL,
    checkedAt: new Date().toISOString(),
    allHealthy: results.every((r) => r.ok),
    results,
  };
}

export interface GitHubOpenPull {
  number: number;
  title: string;
  state: string;
  htmlUrl: string;
  headRef: string;
  baseRef: string;
  userLogin: string;
  createdAt: string;
  mergeableState?: string;
}

export interface GitHubCheckRunSummary {
  id: number;
  name: string;
  status: string;
  conclusion: string | null;
  htmlUrl: string;
  startedAt: string | null;
  completedAt: string | null;
  outputTitle: string | null;
  outputSummary: string | null;
  outputText: string | null;
}

export interface GitHubSyncStatus {
  configured: boolean;
  repo: string;
  baseBranch: string;
  tokenUser: string | null;
  scopes: string[] | null;
  permissions: {
    admin: boolean;
    maintain: boolean;
    push: boolean;
    triage: boolean;
    pull: boolean;
  } | null;
  headSha: string | null;
  headMessage: string | null;
  protectedBranch: boolean | null;
  openPulls: GitHubOpenPull[];
  checkRuns: GitHubCheckRunSummary[];
  error: string | null;
}

export async function getGitHubSyncStatus(): Promise<GitHubSyncStatus> {
  const token = getGitHubToken();
  const baseStatus: GitHubSyncStatus = {
    configured: Boolean(token),
    repo: `${GITHUB_OWNER}/${GITHUB_REPO}`,
    baseBranch: GITHUB_BASE_BRANCH,
    tokenUser: null,
    scopes: null,
    permissions: null,
    headSha: null,
    headMessage: null,
    protectedBranch: null,
    openPulls: [],
    checkRuns: [],
    error: null,
  };

  if (!token) {
    baseStatus.error = 'GITHUB_PAT is not set in the server environment.';
    return baseStatus;
  }

  try {
    const [userRes, repoRes, branchRes, pullsRes, checksRes] = await Promise.all([
      githubRequest<{ login: string }>('/user', { token }),
      githubRequest<{
        permissions?: {
          admin?: boolean;
          maintain?: boolean;
          push?: boolean;
          triage?: boolean;
          pull?: boolean;
        };
      }>(`/repos/${GITHUB_OWNER}/${GITHUB_REPO}`, { token }),
      githubRequest<{
        protected: boolean;
        commit: { sha: string; commit?: { message?: string } };
      }>(`/repos/${GITHUB_OWNER}/${GITHUB_REPO}/branches/${GITHUB_BASE_BRANCH}`, {
        token,
      }),
      githubRequest<
        Array<{
          number: number;
          title: string;
          state: string;
          html_url: string;
          head: { ref: string };
          base: { ref: string };
          user?: { login?: string } | null;
          created_at: string;
        }>
      >(`/repos/${GITHUB_OWNER}/${GITHUB_REPO}/pulls?state=open&per_page=20`, {
        token,
      }),
      githubRequest<{
        check_runs?: Array<{
          id: number;
          name: string;
          status: string;
          conclusion: string | null;
          html_url: string;
          started_at: string | null;
          completed_at: string | null;
          output?: {
            title?: string | null;
            summary?: string | null;
            text?: string | null;
          } | null;
        }>;
      }>(
        `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/commits/${GITHUB_BASE_BRANCH}/check-runs?per_page=20`,
        { token }
      ).catch(() => ({ data: { check_runs: [] }, headers: new Headers() })),
    ]);

    const rawScopes = userRes.headers.get('x-oauth-scopes');
    baseStatus.tokenUser = userRes.data.login || null;
    baseStatus.scopes = rawScopes
      ? rawScopes
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean)
      : null;

    if (repoRes.data.permissions) {
      baseStatus.permissions = {
        admin: Boolean(repoRes.data.permissions.admin),
        maintain: Boolean(repoRes.data.permissions.maintain),
        push: Boolean(repoRes.data.permissions.push),
        triage: Boolean(repoRes.data.permissions.triage),
        pull: Boolean(repoRes.data.permissions.pull),
      };
    }

    baseStatus.headSha = branchRes.data.commit?.sha || null;
    baseStatus.headMessage =
      branchRes.data.commit?.commit?.message?.split('\n')[0] || null;
    baseStatus.protectedBranch = Boolean(branchRes.data.protected);

    baseStatus.openPulls = pullsRes.data.map((pr) => ({
      number: pr.number,
      title: pr.title,
      state: pr.state,
      htmlUrl: pr.html_url,
      headRef: pr.head?.ref || '',
      baseRef: pr.base?.ref || '',
      userLogin: pr.user?.login || 'unknown',
      createdAt: pr.created_at,
    }));

    baseStatus.checkRuns = (checksRes.data.check_runs || []).map((run) => ({
      id: run.id,
      name: run.name,
      status: run.status,
      conclusion: run.conclusion,
      htmlUrl: run.html_url,
      startedAt: run.started_at,
      completedAt: run.completed_at,
      outputTitle: run.output?.title || null,
      outputSummary: run.output?.summary || null,
      outputText: run.output?.text || null,
    }));
  } catch (err) {
    baseStatus.error =
      err instanceof Error ? err.message : 'Unable to query GitHub repository status.';
  }

  return baseStatus;
}

async function fetchMainTreeSnapshot(token: string): Promise<{
  headSha: string;
  baseTreeSha: string;
  remoteBlobs: Map<string, { sha: string; size: number }>;
}> {
  const branchRes = await githubRequest<{
    commit: { sha: string; commit: { tree: { sha: string } } };
  }>(`/repos/${GITHUB_OWNER}/${GITHUB_REPO}/branches/${GITHUB_BASE_BRANCH}`, {
    token,
  });

  const headSha = branchRes.data.commit.sha;
  const baseTreeSha = branchRes.data.commit.commit.tree.sha;

  const treeRes = await githubRequest<{
    truncated: boolean;
    tree: Array<{ path: string; type: string; sha: string; size?: number }>;
  }>(`/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/trees/${baseTreeSha}?recursive=1`, {
    token,
  });

  if (treeRes.data.truncated) {
    throw new Error('Remote Git tree response was truncated by GitHub.');
  }

  const remoteBlobs = new Map<string, { sha: string; size: number }>();
  for (const item of treeRes.data.tree) {
    if (item.type === 'blob') {
      remoteBlobs.set(item.path, {
        sha: item.sha,
        size: typeof item.size === 'number' ? item.size : 0,
      });
    }
  }

  return { headSha, baseTreeSha, remoteBlobs };
}

export async function computeWorkspaceDiff(): Promise<WorkspaceDiffResult> {
  const token = getGitHubToken();
  if (!token) {
    throw new Error('GITHUB_PAT is not configured on the server.');
  }

  const { headSha, remoteBlobs } = await fetchMainTreeSnapshot(token);
  const { rulePatterns, matcher } = buildGitignoreMatcher();
  const baseIndex = loadSyncBaseIndex();
  const localPaths = collectWorkspaceFiles(ROOT_DIR);
  const localPathSet = new Set(localPaths);

  const changed: WorkspaceDiffEntry[] = [];
  const ignoredFiles: IgnoredFileEntry[] = [];
  const matchedCounts = new Map<string, number>();

  for (const relPath of localPaths) {
    const resolved = resolveSafeSyncPath(relPath);
    if (!resolved) continue;
    const content = fs.readFileSync(resolved.absPath);
    const ignoreTest = matcher.test(relPath);
    const remote = remoteBlobs.get(relPath);
    const baseRecord = baseIndex[relPath];

    if (ignoreTest.ignored) {
      const rulePattern = ignoreTest.rule?.pattern || '.gitignore';
      matchedCounts.set(rulePattern, (matchedCounts.get(rulePattern) || 0) + 1);
      ignoredFiles.push({
        path: relPath,
        matchedRule: rulePattern,
        ruleLocked: LOCKED_GITIGNORE_PATTERNS.has(rulePattern),
        existsLocal: true,
        existsRemote: Boolean(remote),
        sizeBytes: content.length,
        remoteSha: remote?.sha,
      });
      continue;
    }

    const localSha = computeGitBlobSha(content);
    const category = classifyPath(relPath);

    if (!remote) {
      const wasOnRemoteBefore = Boolean(baseRecord);
      changed.push({
        path: relPath,
        status: wasOnRemoteBefore ? 'deleted_remote' : 'added_local',
        category,
        sizeBytes: content.length,
        hasBase: wasOnRemoteBefore,
        remoteUpdatedSinceBase: wasOnRemoteBefore,
        canPush: category !== 'superseded',
        canPull: wasOnRemoteBefore,
        canDeleteRemote: false,
      });
    } else if (remote.sha !== localSha) {
      changed.push({
        path: relPath,
        status: 'modified',
        category,
        sizeBytes: content.length,
        remoteSizeBytes: remote.size,
        remoteSha: remote.sha,
        hasBase: Boolean(baseRecord),
        remoteUpdatedSinceBase: Boolean(baseRecord && baseRecord.blobSha !== remote.sha),
        canPush: true,
        canPull: true,
        canDeleteRemote: false,
      });
    } else {
      if (!baseRecord || baseRecord.blobSha !== remote.sha) {
        saveSyncBaseSnapshot(relPath, remote.sha, content, 'seed');
      }
    }
  }

  for (const [remotePath, remote] of remoteBlobs.entries()) {
    if (isHardProtectedPath(remotePath) || isHiddenManagedPath(remotePath)) continue;
    if (localPathSet.has(remotePath)) continue;
    const baseRecord = baseIndex[remotePath];

    if (isLocalUploadSkipped(remotePath)) {
      changed.push({
        path: remotePath,
        status: 'remote_only',
        category: classifyPath(remotePath),
        sizeBytes: remote.size,
        remoteSizeBytes: remote.size,
        remoteSha: remote.sha,
        hasBase: Boolean(baseRecord),
        remoteUpdatedSinceBase: Boolean(baseRecord && baseRecord.blobSha !== remote.sha),
        canPush: false,
        canPull: false,
        canDeleteRemote: true,
      });
      continue;
    }

    const ignoreTest = matcher.test(remotePath);
    if (ignoreTest.ignored) {
      const rulePattern = ignoreTest.rule?.pattern || '.gitignore';
      matchedCounts.set(rulePattern, (matchedCounts.get(rulePattern) || 0) + 1);
      ignoredFiles.push({
        path: remotePath,
        matchedRule: rulePattern,
        ruleLocked: LOCKED_GITIGNORE_PATTERNS.has(rulePattern),
        existsLocal: false,
        existsRemote: true,
        sizeBytes: remote.size,
        remoteSha: remote.sha,
      });
      continue;
    }

    changed.push({
      path: remotePath,
      status: 'remote_only',
      category: classifyPath(remotePath),
      sizeBytes: remote.size,
      remoteSizeBytes: remote.size,
      remoteSha: remote.sha,
      hasBase: Boolean(baseRecord),
      remoteUpdatedSinceBase: Boolean(baseRecord && baseRecord.blobSha !== remote.sha),
      canPush: false,
      canPull: true,
      canDeleteRemote: true,
    });
  }

  changed.sort((a, b) => a.path.localeCompare(b.path));
  ignoredFiles.sort((a, b) => a.path.localeCompare(b.path));

  const rules: GitignoreRuleEntry[] = rulePatterns.map((pattern) => ({
    pattern,
    locked: LOCKED_GITIGNORE_PATTERNS.has(pattern),
    matchedCount: matchedCounts.get(pattern) || 0,
  }));

  return {
    headSha,
    files: changed,
    gitignore: {
      rules,
      ignoredFiles,
    },
    commitSuggestion: buildPreparedCommitSuggestion(changed),
  };
}

function isBufferBinary(buf: Buffer, relPath: string): boolean {
  const ext = path.extname(relPath).toLowerCase();
  if (BINARY_EXTENSIONS.has(ext)) return true;
  const sample = buf.subarray(0, 8192);
  return sample.includes(0);
}

function buildUnifiedLineHunks(
  remoteText: string,
  localText: string
): { diffHunks: DiffHunkLine[]; diffTruncated: boolean } {
  const a = remoteText ? remoteText.split(/\r?\n/) : [];
  const b = localText ? localText.split(/\r?\n/) : [];
  const maxLines = 600;
  const diffHunks: DiffHunkLine[] = [];

  let i = 0;
  let j = 0;
  while ((i < a.length || j < b.length) && diffHunks.length < maxLines) {
    if (i < a.length && j < b.length && a[i] === b[j]) {
      diffHunks.push({ type: 'ctx', oldLine: i + 1, newLine: j + 1, text: a[i] });
      i += 1;
      j += 1;
      continue;
    }

    let foundMatch = false;
    for (let look = 1; look <= 16; look += 1) {
      if (i + look < a.length && j < b.length && a[i + look] === b[j]) {
        for (let k = 0; k < look && diffHunks.length < maxLines; k += 1) {
          diffHunks.push({
            type: 'del',
            oldLine: i + k + 1,
            newLine: null,
            text: a[i + k],
          });
        }
        i += look;
        foundMatch = true;
        break;
      }
      if (j + look < b.length && i < a.length && a[i] === b[j + look]) {
        for (let k = 0; k < look && diffHunks.length < maxLines; k += 1) {
          diffHunks.push({
            type: 'add',
            oldLine: null,
            newLine: j + k + 1,
            text: b[j + k],
          });
        }
        j += look;
        foundMatch = true;
        break;
      }
    }

    if (!foundMatch) {
      if (i < a.length) {
        diffHunks.push({ type: 'del', oldLine: i + 1, newLine: null, text: a[i] });
        i += 1;
      }
      if (j < b.length && diffHunks.length < maxLines) {
        diffHunks.push({ type: 'add', oldLine: null, newLine: j + 1, text: b[j] });
        j += 1;
      }
    }
  }

  return {
    diffHunks,
    diffTruncated: i < a.length || j < b.length,
  };
}

export async function getWorkspaceFileDiff(
  rawPath: string
): Promise<WorkspaceFileDiffDetail> {
  const token = getGitHubToken();
  if (!token) {
    throw new Error('GITHUB_PAT is not configured on the server.');
  }

  const resolved = resolveSafeSyncPath(rawPath);
  if (!resolved) {
    throw new Error(`Invalid or restricted workspace path: ${String(rawPath)}`);
  }

  const { relPath, absPath } = resolved;
  const { remoteBlobs } = await fetchMainTreeSnapshot(token);
  const remoteMeta = remoteBlobs.get(relPath);
  const baseSnapshot = getSyncBaseSnapshot(relPath);

  let existsLocal = false;
  let localBuf = Buffer.alloc(0);
  if (fs.existsSync(absPath) && fs.statSync(absPath).isFile()) {
    existsLocal = true;
    localBuf = fs.readFileSync(absPath);
  }

  let existsRemote = false;
  let remoteBuf = Buffer.alloc(0);
  if (remoteMeta) {
    existsRemote = true;
    const blobRes = await githubRequest<{ content: string; encoding: string }>(
      `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/blobs/${remoteMeta.sha}`,
      { token }
    );
    remoteBuf = Buffer.from(
      blobRes.data.content.replace(/\n/g, ''),
      blobRes.data.encoding === 'base64' ? 'base64' : 'utf8'
    );
  }

  const isBinary =
    (existsLocal && isBufferBinary(localBuf, relPath)) ||
    (existsRemote && isBufferBinary(remoteBuf, relPath)) ||
    Boolean(baseSnapshot && isBufferBinary(baseSnapshot.buffer, relPath));

  const MAX_TEXT_BYTES = 160_000;
  let truncated = false;

  let localContent: string | null = null;
  let remoteContent: string | null = null;
  let baseContent: string | null = null;

  if (!isBinary) {
    if (existsLocal) {
      if (localBuf.length > MAX_TEXT_BYTES) {
        truncated = true;
        localContent = localBuf.subarray(0, MAX_TEXT_BYTES).toString('utf8');
      } else {
        localContent = localBuf.toString('utf8');
      }
    }
    if (existsRemote) {
      if (remoteBuf.length > MAX_TEXT_BYTES) {
        truncated = true;
        remoteContent = remoteBuf.subarray(0, MAX_TEXT_BYTES).toString('utf8');
      } else {
        remoteContent = remoteBuf.toString('utf8');
      }
    }
    if (baseSnapshot) {
      if (baseSnapshot.buffer.length > MAX_TEXT_BYTES) {
        truncated = true;
        baseContent = baseSnapshot.buffer.subarray(0, MAX_TEXT_BYTES).toString('utf8');
      } else {
        baseContent = baseSnapshot.buffer.toString('utf8');
      }
    }
  }

  const linePreview = isBinary
    ? { diffHunks: [], diffTruncated: false }
    : buildUnifiedLineHunks(remoteContent ?? '', localContent ?? '');

  return {
    path: relPath,
    existsLocal,
    existsRemote,
    remoteSha: remoteMeta?.sha || null,
    hasBase: Boolean(baseSnapshot),
    baseSha: baseSnapshot?.blobSha || null,
    remoteUpdatedSinceBase: Boolean(
      baseSnapshot && remoteMeta && baseSnapshot.blobSha !== remoteMeta.sha
    ),
    isBinary,
    localSize: localBuf.length,
    remoteSize: remoteBuf.length,
    localContent,
    remoteContent,
    baseContent,
    diffHunks: linePreview.diffHunks,
    diffTruncated: linePreview.diffTruncated,
    truncated,
  };
}

export interface PullFilesParams {
  paths: string[];
  expectedHeadSha?: string;
  overwritePaths?: string[];
}

export interface PullFilesResult {
  headSha: string;
  filesPulled: number;
  paths: string[];
  autoMergedPaths: string[];
  keptLocalPaths: string[];
  deletedLocalPaths: string[];
  conflicts: MergeConflictEntry[];
}

export async function pullFilesFromGitHub(
  params: PullFilesParams
): Promise<PullFilesResult> {
  const token = getGitHubToken();
  if (!token) {
    throw new Error('GITHUB_PAT is not configured on the server.');
  }

  if (!Array.isArray(params.paths) || params.paths.length === 0) {
    throw new Error('Select at least one file to pull from main.');
  }
  if (params.paths.length > 200) {
    throw new Error('Too many files selected in a single pull batch (max 200).');
  }

  const { headSha, remoteBlobs } = await fetchMainTreeSnapshot(token);
  if (params.expectedHeadSha && params.expectedHeadSha !== headSha) {
    throw new GitHubSyncConflictError(
      `main moved to ${headSha.slice(0, 8)} since your last refresh. Click Refresh Diff before pulling.`,
      { headSha }
    );
  }

  const overwriteSet = new Set(
    (Array.isArray(params.overwritePaths) ? params.overwritePaths : []).map(
      normalizeRelativePath
    )
  );
  const targets: Array<{ relPath: string; absPath: string; sha: string }> = [];
  const deletedTargets: Array<{ relPath: string; absPath: string; baseSha: string; baseBuf: Buffer }> = [];
  const seen = new Set<string>();

  for (const rawPath of params.paths) {
    const resolved = resolveSafeSyncPath(rawPath);
    if (!resolved || isLocalUploadSkipped(resolved.relPath)) {
      throw new Error(`Invalid or blocked pull file path: ${String(rawPath)}`);
    }
    if (seen.has(resolved.relPath)) continue;
    const remote = remoteBlobs.get(resolved.relPath);
    if (!remote) {
      const baseSnapshot = getSyncBaseSnapshot(resolved.relPath);
      if (baseSnapshot) {
        seen.add(resolved.relPath);
        deletedTargets.push({
          relPath: resolved.relPath,
          absPath: resolved.absPath,
          baseSha: baseSnapshot.blobSha,
          baseBuf: baseSnapshot.buffer,
        });
        continue;
      }
      throw new Error(`File does not exist on main: ${resolved.relPath}`);
    }
    seen.add(resolved.relPath);
    targets.push({ relPath: resolved.relPath, absPath: resolved.absPath, sha: remote.sha });
  }

  const fetched = await runWithConcurrency(targets, 6, async (item) => {
    const blobRes = await githubRequest<{ content: string; encoding: string }>(
      `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/blobs/${item.sha}`,
      { token }
    );
    const buffer = Buffer.from(
      blobRes.data.content.replace(/\n/g, ''),
      blobRes.data.encoding === 'base64' ? 'base64' : 'utf8'
    );
    return { ...item, buffer };
  });

  const pulledPaths: string[] = [];
  const autoMergedPaths: string[] = [];
  const keptLocalPaths: string[] = [];
  const deletedLocalPaths: string[] = [];
  const conflicts: MergeConflictEntry[] = [];

  for (const delTarget of deletedTargets) {
    const existsLocal =
      fs.existsSync(delTarget.absPath) && fs.statSync(delTarget.absPath).isFile();
    if (!existsLocal) {
      removeSyncBaseSnapshot(delTarget.relPath);
      continue;
    }

    const localBuf = fs.readFileSync(delTarget.absPath);
    const localSha = computeGitBlobSha(localBuf);
    if (overwriteSet.has(delTarget.relPath) || localSha === delTarget.baseSha) {
      fs.unlinkSync(delTarget.absPath);
      removeSyncBaseSnapshot(delTarget.relPath);
      deletedLocalPaths.push(delTarget.relPath);
      continue;
    }

    const isBinary =
      isBufferBinary(localBuf, delTarget.relPath) ||
      isBufferBinary(delTarget.baseBuf, delTarget.relPath);
    conflicts.push({
      path: delTarget.relPath,
      direction: 'pull',
      reason: 'deleted_remote',
      remoteSha: '',
      baseSha: delTarget.baseSha,
      isBinary,
      baseContent: isBinary ? null : delTarget.baseBuf.toString('utf8'),
      localContent: isBinary ? null : localBuf.toString('utf8'),
      remoteContent: null,
      suggestedContent: null,
    });
  }

  for (const file of fetched) {
    const parentDir = path.dirname(file.absPath);
    fs.mkdirSync(parentDir, { recursive: true });
    const existsLocal =
      fs.existsSync(file.absPath) && fs.statSync(file.absPath).isFile();

    if (!existsLocal || overwriteSet.has(file.relPath)) {
      fs.writeFileSync(file.absPath, file.buffer);
      saveSyncBaseSnapshot(file.relPath, file.sha, file.buffer, 'pull');
      pulledPaths.push(file.relPath);
      continue;
    }

    const localBuf = fs.readFileSync(file.absPath);
    const localSha = computeGitBlobSha(localBuf);
    if (localSha === file.sha) {
      saveSyncBaseSnapshot(file.relPath, file.sha, file.buffer, 'seed');
      keptLocalPaths.push(file.relPath);
      continue;
    }

    const baseSnapshot = getSyncBaseSnapshot(file.relPath);
    const isBinary =
      isBufferBinary(localBuf, file.relPath) ||
      isBufferBinary(file.buffer, file.relPath) ||
      Boolean(baseSnapshot && isBufferBinary(baseSnapshot.buffer, file.relPath));

    if (baseSnapshot && !isBinary) {
      if (baseSnapshot.blobSha === file.sha) {
        keptLocalPaths.push(file.relPath);
        continue;
      }

      const localText = localBuf.toString('utf8');
      const baseText = baseSnapshot.buffer.toString('utf8');
      const remoteText = file.buffer.toString('utf8');
      const merged = runThreeWayMergeText(localText, baseText, remoteText);

      if (!merged.hasConflict) {
        fs.writeFileSync(file.absPath, Buffer.from(merged.text, 'utf8'));
        saveSyncBaseSnapshot(file.relPath, file.sha, file.buffer, 'merge');
        pulledPaths.push(file.relPath);
        autoMergedPaths.push(file.relPath);
      } else {
        conflicts.push({
          path: file.relPath,
          direction: 'pull',
          reason: 'overlap',
          remoteSha: file.sha,
          baseSha: baseSnapshot.blobSha,
          isBinary: false,
          baseContent: baseText,
          localContent: localText,
          remoteContent: remoteText,
          suggestedContent: merged.text,
        });
      }
      continue;
    }

    conflicts.push({
      path: file.relPath,
      direction: 'pull',
      reason: isBinary ? 'binary' : 'no_base',
      remoteSha: file.sha,
      baseSha: baseSnapshot?.blobSha || null,
      isBinary,
      baseContent:
        baseSnapshot && !isBinary ? baseSnapshot.buffer.toString('utf8') : null,
      localContent: isBinary ? null : localBuf.toString('utf8'),
      remoteContent: isBinary ? null : file.buffer.toString('utf8'),
      suggestedContent: null,
    });
  }

  return {
    headSha,
    filesPulled: pulledPaths.length,
    paths: pulledPaths,
    autoMergedPaths,
    keptLocalPaths,
    deletedLocalPaths,
    conflicts,
  };
}

export interface ResolveMergeParams {
  path: string;
  resolvedContent?: string;
  remoteSha?: string;
  action?: 'save' | 'delete_local' | 'keep_local';
}

export async function resolveWorkspaceMerge(
  params: ResolveMergeParams
): Promise<{ path: string; baseSha: string | null }> {
  const token = getGitHubToken();
  if (!token) {
    throw new Error('GITHUB_PAT is not configured on the server.');
  }
  const resolved = resolveSafeSyncPath(params.path);
  if (!resolved || isLocalUploadSkipped(resolved.relPath)) {
    throw new Error(`Invalid or blocked merge file path: ${String(params.path)}`);
  }

  const action = params.action || 'save';
  if (action === 'delete_local') {
    if (fs.existsSync(resolved.absPath) && fs.statSync(resolved.absPath).isFile()) {
      fs.unlinkSync(resolved.absPath);
    }
    removeSyncBaseSnapshot(resolved.relPath);
    return { path: resolved.relPath, baseSha: null };
  }

  if (action === 'keep_local') {
    if (typeof params.resolvedContent === 'string') {
      fs.mkdirSync(path.dirname(resolved.absPath), { recursive: true });
      fs.writeFileSync(resolved.absPath, Buffer.from(params.resolvedContent, 'utf8'));
    }
    removeSyncBaseSnapshot(resolved.relPath);
    return { path: resolved.relPath, baseSha: null };
  }

  if (typeof params.resolvedContent !== 'string') {
    throw new Error('Resolved file content is required.');
  }
  if (typeof params.remoteSha !== 'string' || !/^[a-f0-9]{40}$/i.test(params.remoteSha)) {
    throw new Error('Valid remote blob SHA is required.');
  }

  const blobRes = await githubRequest<{ content: string; encoding: string }>(
    `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/blobs/${params.remoteSha}`,
    { token }
  );
  const remoteBuf = Buffer.from(
    blobRes.data.content.replace(/\n/g, ''),
    blobRes.data.encoding === 'base64' ? 'base64' : 'utf8'
  );

  fs.mkdirSync(path.dirname(resolved.absPath), { recursive: true });
  fs.writeFileSync(resolved.absPath, Buffer.from(params.resolvedContent, 'utf8'));
  saveSyncBaseSnapshot(resolved.relPath, params.remoteSha, remoteBuf, 'merge');

  return { path: resolved.relPath, baseSha: params.remoteSha };
}

function sanitizeBranchName(rawBranch: string): string {
  const cleaned = rawBranch.trim();
  if (
    !cleaned ||
    cleaned.length > 100 ||
    cleaned === GITHUB_BASE_BRANCH ||
    cleaned.startsWith('/') ||
    cleaned.endsWith('/') ||
    cleaned.includes('..') ||
    cleaned.includes('//') ||
    !/^[A-Za-z0-9._/-]+$/.test(cleaned)
  ) {
    throw new Error(
      'Branch name must be a valid Git branch name and cannot be "main".'
    );
  }
  return cleaned;
}

export interface SyncWorkspaceParams {
  mode: 'pr' | 'direct';
  paths: string[];
  deletePaths?: string[];
  commitMessage: string;
  branchName?: string;
  prTitle?: string;
  prBody?: string;
  expectedHeadSha?: string;
  knownRemoteShas?: Record<string, string>;
}

export interface SyncWorkspaceResult {
  mode: 'pr' | 'direct';
  commitSha: string;
  commitUrl: string;
  branchName: string;
  filesSynced: number;
  filesDeleted: number;
  pullNumber?: number;
  pullUrl?: string;
}

export async function syncWorkspaceToGitHub(
  params: SyncWorkspaceParams
): Promise<SyncWorkspaceResult> {
  const token = getGitHubToken();
  if (!token) {
    throw new Error('GITHUB_PAT is not configured on the server.');
  }

  if (params.mode !== 'pr' && params.mode !== 'direct') {
    throw new Error('Sync mode must be "pr" or "direct".');
  }

  const commitMessage = (params.commitMessage || '').trim().slice(0, 4000);
  if (!commitMessage) {
    throw new Error('Commit message is required.');
  }

  const uploadInputs = Array.isArray(params.paths) ? params.paths : [];
  const deleteInputs = Array.isArray(params.deletePaths) ? params.deletePaths : [];

  if (uploadInputs.length === 0 && deleteInputs.length === 0) {
    throw new Error('Select at least one file to push or remove.');
  }
  if (uploadInputs.length + deleteInputs.length > 250) {
    throw new Error('Too many files selected in a single sync batch (max 250).');
  }

  const resolvedFiles: Array<{ relPath: string; content: Buffer }> = [];
  const seenUpload = new Set<string>();
  for (const rawPath of uploadInputs) {
    const resolved = resolveSafeSyncPath(rawPath);
    if (
      !resolved ||
      isLocalUploadSkipped(resolved.relPath) ||
      !fs.existsSync(resolved.absPath) ||
      !fs.statSync(resolved.absPath).isFile()
    ) {
      throw new Error(`Invalid or blocked workspace file path: ${String(rawPath)}`);
    }
    if (seenUpload.has(resolved.relPath)) continue;
    seenUpload.add(resolved.relPath);
    resolvedFiles.push({
      relPath: resolved.relPath,
      content: fs.readFileSync(resolved.absPath),
    });
  }

  const { headSha, baseTreeSha, remoteBlobs } = await fetchMainTreeSnapshot(token);
  if (params.expectedHeadSha && params.expectedHeadSha !== headSha) {
    throw new GitHubSyncConflictError(
      `main moved to ${headSha.slice(0, 8)} since your last refresh. Refresh Diff and merge any remote updates before pushing.`,
      { headSha }
    );
  }

  const knownRemoteShas =
    params.knownRemoteShas && typeof params.knownRemoteShas === 'object'
      ? params.knownRemoteShas
      : {};
  const remoteAheadFiles: Array<{
    file: { relPath: string; content: Buffer };
    remoteSha: string;
    baseSnapshot: ReturnType<typeof getSyncBaseSnapshot>;
    reasonHint: 'remote_ahead' | 'no_base_changed';
  }> = [];

  const remoteDeletedFiles: Array<{
    file: { relPath: string; content: Buffer };
    baseSnapshot: NonNullable<ReturnType<typeof getSyncBaseSnapshot>>;
  }> = [];

  for (const file of resolvedFiles) {
    const remote = remoteBlobs.get(file.relPath);
    const baseSnapshot = getSyncBaseSnapshot(file.relPath);
    if (!remote) {
      if (baseSnapshot) {
        remoteDeletedFiles.push({ file, baseSnapshot });
      }
      continue;
    }
    if (baseSnapshot && baseSnapshot.blobSha !== remote.sha) {
      remoteAheadFiles.push({
        file,
        remoteSha: remote.sha,
        baseSnapshot,
        reasonHint: 'remote_ahead',
      });
    } else if (
      !baseSnapshot &&
      knownRemoteShas[file.relPath] &&
      knownRemoteShas[file.relPath] !== remote.sha
    ) {
      remoteAheadFiles.push({
        file,
        remoteSha: remote.sha,
        baseSnapshot: null,
        reasonHint: 'no_base_changed',
      });
    }
  }

  if (remoteAheadFiles.length > 0 || remoteDeletedFiles.length > 0) {
    const aheadConflicts = await runWithConcurrency(remoteAheadFiles, 6, async (item) => {
      const blobRes = await githubRequest<{ content: string; encoding: string }>(
        `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/blobs/${item.remoteSha}`,
        { token }
      );
      const remoteBuf = Buffer.from(
        blobRes.data.content.replace(/\n/g, ''),
        blobRes.data.encoding === 'base64' ? 'base64' : 'utf8'
      );
      const isBinary =
        isBufferBinary(item.file.content, item.file.relPath) ||
        isBufferBinary(remoteBuf, item.file.relPath) ||
        Boolean(item.baseSnapshot && isBufferBinary(item.baseSnapshot.buffer, item.file.relPath));

      if (item.baseSnapshot && !isBinary) {
        const localText = item.file.content.toString('utf8');
        const baseText = item.baseSnapshot.buffer.toString('utf8');
        const remoteText = remoteBuf.toString('utf8');
        const merged = runThreeWayMergeText(localText, baseText, remoteText);
        return {
          path: item.file.relPath,
          direction: 'push' as const,
          reason: merged.hasConflict
            ? ('overlap' as const)
            : ('remote_ahead_clean' as const),
          remoteSha: item.remoteSha,
          baseSha: item.baseSnapshot.blobSha,
          isBinary: false,
          baseContent: baseText,
          localContent: localText,
          remoteContent: remoteText,
          suggestedContent: merged.text,
        };
      }

      return {
        path: item.file.relPath,
        direction: 'push' as const,
        reason: isBinary ? ('binary' as const) : ('no_base' as const),
        remoteSha: item.remoteSha,
        baseSha: item.baseSnapshot?.blobSha || null,
        isBinary,
        baseContent:
          item.baseSnapshot && !isBinary
            ? item.baseSnapshot.buffer.toString('utf8')
            : null,
        localContent: isBinary ? null : item.file.content.toString('utf8'),
        remoteContent: isBinary ? null : remoteBuf.toString('utf8'),
        suggestedContent: null,
      };
    });

    const deletedConflicts: MergeConflictEntry[] = remoteDeletedFiles.map((item) => {
      const isBinary =
        isBufferBinary(item.file.content, item.file.relPath) ||
        isBufferBinary(item.baseSnapshot.buffer, item.file.relPath);
      return {
        path: item.file.relPath,
        direction: 'push',
        reason: 'deleted_remote',
        remoteSha: '',
        baseSha: item.baseSnapshot.blobSha,
        isBinary,
        baseContent: isBinary ? null : item.baseSnapshot.buffer.toString('utf8'),
        localContent: isBinary ? null : item.file.content.toString('utf8'),
        remoteContent: null,
        suggestedContent: null,
      };
    });

    const conflicts = [...aheadConflicts, ...deletedConflicts];
    throw new GitHubSyncConflictError(
      `${conflicts.length} checked file(s) changed or were deleted on GitHub main since their last synced baseline. Review and resolve below before pushing.`,
      { conflicts, headSha }
    );
  }

  const resolvedDeletePaths: string[] = [];
  const seenDelete = new Set<string>();
  for (const rawPath of deleteInputs) {
    const resolved = resolveSafeSyncPath(rawPath);
    if (!resolved) {
      throw new Error(`Invalid or blocked remote deletion path: ${String(rawPath)}`);
    }
    if (seenUpload.has(resolved.relPath)) {
      throw new Error(`Cannot both upload and delete the same path: ${resolved.relPath}`);
    }
    if (seenDelete.has(resolved.relPath)) continue;
    if (!remoteBlobs.has(resolved.relPath)) {
      throw new Error(`Remote file does not exist on main: ${resolved.relPath}`);
    }
    seenDelete.add(resolved.relPath);
    resolvedDeletePaths.push(resolved.relPath);
  }

  const defaultBranch = `aistudio/sync-${new Date()
    .toISOString()
    .replace(/[-:T]/g, '')
    .slice(0, 14)}`;
  const targetBranch =
    params.mode === 'direct'
      ? GITHUB_BASE_BRANCH
      : sanitizeBranchName(params.branchName || defaultBranch);

  const uploadedTreeEntries = await runWithConcurrency(
    resolvedFiles,
    6,
    async (file) => {
      const blobRes = await githubRequest<{ sha: string }>(
        `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/blobs`,
        {
          method: 'POST',
          token,
          body: {
            content: file.content.toString('base64'),
            encoding: 'base64',
          },
        }
      );
      return {
        path: file.relPath,
        mode: '100644' as const,
        type: 'blob' as const,
        sha: blobRes.data.sha as string | null,
        rawContent: file.content,
        uploadedSha: blobRes.data.sha,
      };
    }
  );

  const deleteTreeEntries = resolvedDeletePaths.map((relPath) => ({
    path: relPath,
    mode: '100644' as const,
    type: 'blob' as const,
    sha: null as string | null,
  }));

  const newTreeRes = await githubRequest<{ sha: string }>(
    `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/trees`,
    {
      method: 'POST',
      token,
      body: {
        base_tree: baseTreeSha,
        tree: [
          ...uploadedTreeEntries.map(({ path: p, mode, type, sha }) => ({
            path: p,
            mode,
            type,
            sha,
          })),
          ...deleteTreeEntries,
        ],
      },
    }
  );

  const newCommitRes = await githubRequest<{ sha: string; html_url: string }>(
    `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/commits`,
    {
      method: 'POST',
      token,
      body: {
        message: commitMessage,
        tree: newTreeRes.data.sha,
        parents: [headSha],
      },
    }
  );

  const commitSha = newCommitRes.data.sha;
  const commitUrl =
    newCommitRes.data.html_url ||
    `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/commit/${commitSha}`;

  if (params.mode === 'direct') {
    await githubRequest(
      `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/refs/heads/${GITHUB_BASE_BRANCH}`,
      {
        method: 'PATCH',
        token,
        body: { sha: commitSha, force: false },
      }
    );

    for (const item of uploadedTreeEntries) {
      saveSyncBaseSnapshot(item.path, item.uploadedSha, item.rawContent, 'push');
    }
    for (const delPath of resolvedDeletePaths) {
      removeSyncBaseSnapshot(delPath);
    }
    clearPendingCommitMeta();

    return {
      mode: 'direct',
      commitSha,
      commitUrl,
      branchName: GITHUB_BASE_BRANCH,
      filesSynced: resolvedFiles.length,
      filesDeleted: resolvedDeletePaths.length,
    };
  }

  await githubRequest(`/repos/${GITHUB_OWNER}/${GITHUB_REPO}/git/refs`, {
    method: 'POST',
    token,
    body: {
      ref: `refs/heads/${targetBranch}`,
      sha: commitSha,
    },
  });

  const firstCommitLine = commitMessage.split('\n')[0];
  const commitExtraLines = commitMessage.split('\n').slice(1).join('\n').trim();
  const prTitle = (params.prTitle || firstCommitLine).trim().slice(0, 240);
  const summaryLines = [
    ...resolvedFiles.map((f) => `- Updated/added \`${f.relPath}\``),
    ...resolvedDeletePaths.map((p) => `- Removed superseded \`${p}\``),
  ];
  const defaultBody = commitExtraLines
    ? `${commitExtraLines}\n\n---\nSynced ${resolvedFiles.length} updated/added file(s) and removed ${resolvedDeletePaths.length} file(s):\n\n${summaryLines.join('\n')}`
    : `Synced ${resolvedFiles.length} updated/added file(s) and removed ${resolvedDeletePaths.length} file(s) from ExoNovel workspace:\n\n${summaryLines.join('\n')}`;
  const prBody = ((params.prBody || '').trim() || defaultBody).slice(0, 6000);

  const prRes = await githubRequest<{ number: number; html_url: string }>(
    `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/pulls`,
    {
      method: 'POST',
      token,
      body: {
        title: prTitle,
        head: targetBranch,
        base: GITHUB_BASE_BRANCH,
        body: prBody,
      },
    }
  );

  clearPendingCommitMeta();

  return {
    mode: 'pr',
    commitSha,
    commitUrl,
    branchName: targetBranch,
    filesSynced: resolvedFiles.length,
    filesDeleted: resolvedDeletePaths.length,
    pullNumber: prRes.data.number,
    pullUrl: prRes.data.html_url,
  };
}

export async function mergeGitHubPullRequest(params: {
  pullNumber: number;
  mergeMethod?: 'merge';
}): Promise<{ merged: boolean; sha: string | null; message: string }> {
  const token = getGitHubToken();
  if (!token) {
    throw new Error('GITHUB_PAT is not configured on the server.');
  }

  const pullNumber = Number(params.pullNumber);
  if (!Number.isInteger(pullNumber) || pullNumber <= 0) {
    throw new Error('Valid pull request number is required.');
  }

  const res = await githubRequest<{
    merged?: boolean;
    sha?: string;
    message?: string;
  }>(`/repos/${GITHUB_OWNER}/${GITHUB_REPO}/pulls/${pullNumber}/merge`, {
    method: 'PUT',
    token,
    body: { merge_method: 'merge' },
  });

  return {
    merged: Boolean(res.data.merged),
    sha: res.data.sha || null,
    message: res.data.message || 'Pull request merged.',
  };
}
