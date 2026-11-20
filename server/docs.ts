import fs from 'node:fs';
import path from 'node:path';

const ROOT_DIR = process.cwd();
const DOCS_DIR = path.join(ROOT_DIR, 'docs');
const AGENTS_FILE = path.join(ROOT_DIR, 'AGENTS.md');

export interface DocHeading {
  id: string;
  text: string;
  level: number;
}

export interface DocNode {
  id: string;
  name: string;
  title: string;
  path: string;
  type: 'file' | 'folder';
  updatedAt?: string;
  wordCount?: number;
  headings?: DocHeading[];
  children?: DocNode[];
}

function slugifyHeading(text: string): string {
  return text
    .toLowerCase()
    .trim()
    .replace(/[`*_~()[\]{}]/g, '')
    .replace(/[^\w\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-');
}

export function extractTitleAndHeadings(
  markdown: string,
  fallbackName: string
): {
  title: string;
  headings: DocHeading[];
  wordCount: number;
} {
  const lines = markdown.split(/\r?\n/);
  let title = fallbackName.replace(/\.md$/i, '');
  let foundH1 = false;
  let inFence = false;
  const headings: DocHeading[] = [];
  const slugCounts = new Map<string, number>();

  for (const line of lines) {
    if (/^\s*(```|~~~)/.test(line)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;

    const match = /^(#{1,4})\s+(.+?)\s*#*\s*$/.exec(line);
    if (!match) continue;

    const level = match[1].length;
    const rawText = match[2].replace(/[*_`]/g, '').trim();
    if (!rawText) continue;

    if (level === 1 && !foundH1) {
      title = rawText;
      foundH1 = true;
    }

    if (level >= 2 && level <= 3) {
      const baseSlug = slugifyHeading(rawText);
      const prevCount = slugCounts.get(baseSlug) ?? 0;
      slugCounts.set(baseSlug, prevCount + 1);
      const id = prevCount === 0 ? baseSlug : `${baseSlug}-${prevCount}`;
      headings.push({ id, text: rawText, level });
    }
  }

  const plainWords = markdown
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/[#>*`_\-|]/g, ' ')
    .trim()
    .split(/\s+/)
    .filter(Boolean);

  return {
    title,
    headings,
    wordCount: plainWords.length,
  };
}

function formatFolderLabel(name: string): string {
  if (name.toLowerCase() === 'adr') return 'ADRs';
  const stripped = name.replace(/^\d+-/, '');
  const prefixMatch = /^(\d+)-/.exec(name);
  const label = stripped
    .split(/[-_]+/)
    .map((part) => (part ? part[0].toUpperCase() + part.slice(1) : ''))
    .join(' ');
  return prefixMatch ? `${prefixMatch[1]} ${label}` : label;
}

function buildFileNode(absPath: string, relPath: string): DocNode {
  const stat = fs.statSync(absPath);
  const content = fs.readFileSync(absPath, 'utf-8');
  const fileName = path.basename(absPath);
  const { title, headings, wordCount } = extractTitleAndHeadings(content, fileName);

  return {
    id: relPath,
    name: fileName,
    title,
    path: relPath,
    type: 'file',
    updatedAt: stat.mtime.toISOString(),
    wordCount,
    headings,
  };
}

function sortNodes(nodes: DocNode[]): DocNode[] {
  return nodes.sort((a, b) => {
    const stemA = a.name.replace(/\.md$/i, '');
    const stemB = b.name.replace(/\.md$/i, '');
    const cmp = stemA.localeCompare(stemB, undefined, { numeric: true });
    if (cmp !== 0) return cmp;
    if (a.type === b.type) return 0;
    // Place parent story ADR file (e.g. 001-auth.md) directly above its matching subfolder (001-auth/)
    return a.type === 'file' ? -1 : 1;
  });
}

function walkDocsDirectory(absDir: string, relDir: string): DocNode[] {
  if (!fs.existsSync(absDir)) return [];

  const entries = fs.readdirSync(absDir, { withFileTypes: true });
  const collected: DocNode[] = [];

  for (const entry of entries) {
    if (entry.name.startsWith('.')) continue;

    const entryAbs = path.join(absDir, entry.name);
    const entryRel = relDir ? `${relDir}/${entry.name}` : entry.name;

    if (entry.isDirectory()) {
      const children = walkDocsDirectory(entryAbs, entryRel);
      if (children.length > 0) {
        collected.push({
          id: entryRel,
          name: entry.name,
          title: formatFolderLabel(entry.name),
          path: entryRel,
          type: 'folder',
          children,
        });
      }
    } else if (entry.isFile() && entry.name.toLowerCase().endsWith('.md')) {
      collected.push(buildFileNode(entryAbs, entryRel));
    }
  }

  return sortNodes(collected);
}

export function getDocsTree(): DocNode[] {
  const nodes: DocNode[] = [];

  if (fs.existsSync(DOCS_DIR)) {
    nodes.push(...walkDocsDirectory(DOCS_DIR, 'docs'));
  }

  if (fs.existsSync(AGENTS_FILE)) {
    nodes.push(buildFileNode(AGENTS_FILE, 'AGENTS.md'));
  }

  return nodes;
}

export function resolveSafeDocPath(requestedPath: string): string | null {
  const normalized = requestedPath.replace(/^\/+/, '').replace(/\\/g, '/');
  if (!normalized || normalized.includes('..')) return null;

  const absTarget = path.resolve(ROOT_DIR, normalized);

  if (absTarget === AGENTS_FILE && fs.existsSync(AGENTS_FILE)) {
    return absTarget;
  }

  const docsPrefix = DOCS_DIR + path.sep;
  if (
    absTarget.startsWith(docsPrefix) &&
    absTarget.toLowerCase().endsWith('.md') &&
    fs.existsSync(absTarget) &&
    fs.statSync(absTarget).isFile()
  ) {
    return absTarget;
  }

  return null;
}
