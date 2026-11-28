import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { mergeDigIn } from 'node-diff3';

const ROOT_DIR = process.cwd();
const SYNC_BASE_DIR = path.join(ROOT_DIR, '.aistudio', 'sync-base');
const SYNC_BASE_BLOBS_DIR = path.join(SYNC_BASE_DIR, 'blobs');
const SYNC_BASE_INDEX_FILE = path.join(SYNC_BASE_DIR, 'index.json');

export type SyncBaseSource = 'seed' | 'pull' | 'push' | 'merge';

export interface SyncBaseRecord {
  blobSha: string;
  updatedAt: string;
  source: SyncBaseSource;
}

export interface SyncBaseSnapshot {
  relPath: string;
  blobSha: string;
  buffer: Buffer;
  source: SyncBaseSource;
  updatedAt: string;
}

export interface ThreeWayMergeTextResult {
  text: string;
  hasConflict: boolean;
}

function normalizeRelPath(relPath: string): string {
  return relPath.replace(/\\/g, '/').replace(/^\/+/, '');
}

function getBlobStoragePath(relPath: string): string {
  const key = crypto
    .createHash('sha256')
    .update(normalizeRelPath(relPath), 'utf8')
    .digest('hex');
  const resolved = path.resolve(SYNC_BASE_BLOBS_DIR, `${key}.bin`);
  if (!resolved.startsWith(SYNC_BASE_BLOBS_DIR + path.sep)) {
    throw new Error('Invalid baseline snapshot storage path.');
  }
  return resolved;
}

function ensureStorageDirs(): void {
  fs.mkdirSync(SYNC_BASE_BLOBS_DIR, { recursive: true, mode: 0o700 });
}

export function loadSyncBaseIndex(): Record<string, SyncBaseRecord> {
  try {
    if (!fs.existsSync(SYNC_BASE_INDEX_FILE)) {
      return {};
    }
    const raw = fs.readFileSync(SYNC_BASE_INDEX_FILE, 'utf8');
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return {};
    }
    return parsed as Record<string, SyncBaseRecord>;
  } catch {
    return {};
  }
}

function writeSyncBaseIndex(index: Record<string, SyncBaseRecord>): void {
  ensureStorageDirs();
  fs.writeFileSync(SYNC_BASE_INDEX_FILE, `${JSON.stringify(index, null, 2)}\n`, {
    encoding: 'utf8',
    mode: 0o600,
  });
}

export function getSyncBaseSnapshot(relPath: string): SyncBaseSnapshot | null {
  const normalized = normalizeRelPath(relPath);
  const index = loadSyncBaseIndex();
  const entry = index[normalized];
  if (!entry || typeof entry.blobSha !== 'string' || !entry.blobSha) {
    return null;
  }

  try {
    const blobPath = getBlobStoragePath(normalized);
    if (!fs.existsSync(blobPath)) {
      return null;
    }
    const buffer = fs.readFileSync(blobPath);
    return {
      relPath: normalized,
      blobSha: entry.blobSha,
      buffer,
      source: entry.source || 'seed',
      updatedAt: entry.updatedAt || '',
    };
  } catch {
    return null;
  }
}

export function saveSyncBaseSnapshot(
  relPath: string,
  blobSha: string,
  buffer: Buffer,
  source: SyncBaseSource
): void {
  const normalized = normalizeRelPath(relPath);
  if (!normalized || !/^[a-f0-9]{40}$/i.test(blobSha)) {
    return;
  }
  ensureStorageDirs();
  const blobPath = getBlobStoragePath(normalized);
  const index = loadSyncBaseIndex();
  const existing = index[normalized];

  if (existing && existing.blobSha === blobSha && fs.existsSync(blobPath)) {
    return;
  }

  fs.writeFileSync(blobPath, buffer, { mode: 0o600 });
  index[normalized] = {
    blobSha,
    source,
    updatedAt: new Date().toISOString(),
  };
  writeSyncBaseIndex(index);
}

export function removeSyncBaseSnapshot(relPath: string): void {
  const normalized = normalizeRelPath(relPath);
  if (!normalized) return;
  const index = loadSyncBaseIndex();
  if (normalized in index) {
    delete index[normalized];
    writeSyncBaseIndex(index);
  }
  try {
    const blobPath = getBlobStoragePath(normalized);
    if (fs.existsSync(blobPath)) {
      fs.unlinkSync(blobPath);
    }
  } catch {
    // Ignore cleanup errors
  }
}

function normalizeLineEndings(text: string): string {
  return text.replace(/\r\n/g, '\n');
}

export function runThreeWayMergeText(
  localText: string,
  baseText: string,
  remoteText: string
): ThreeWayMergeTextResult {
  const preferCrlf = localText.includes('\r\n') && !/[^\r]\n/.test(localText);
  const localNorm = normalizeLineEndings(localText);
  const baseNorm = normalizeLineEndings(baseText);
  const remoteNorm = normalizeLineEndings(remoteText);

  const merged = mergeDigIn(
    localNorm.split('\n'),
    baseNorm.split('\n'),
    remoteNorm.split('\n'),
    {
      excludeFalseConflicts: true,
      label: {
        a: 'Local Container',
        o: 'Last Synced Base',
        b: 'GitHub main',
      },
    }
  );

  let joined = merged.result.join('\n');
  if (preferCrlf) {
    joined = joined.replace(/\n/g, '\r\n');
  }

  return {
    text: joined,
    hasConflict: Boolean(merged.conflict),
  };
}
