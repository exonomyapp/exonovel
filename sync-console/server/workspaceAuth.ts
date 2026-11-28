import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import type { NextFunction, Request, Response } from 'express';
import { Pool } from 'pg';

const WORKSPACE_SIGNING_KEY_FILE = path.join(
  process.cwd(),
  '.aistudio',
  '.ws-signing.key'
);
const SESSION_COOKIE_NAME = 'exonovel_ws_session';
const SESSION_MAX_AGE_SECONDS = 60 * 60 * 24 * 7; // 7 days
const SYNC_REQUESTED_WITH = 'exonovel-sync';

const MAX_FAILED_UNLOCKS = 5;
const UNLOCK_LOCKOUT_MS = 5 * 60 * 1000;

let cachedSigningKey: Buffer | null = null;
let cachedEnvPasscodeRaw: string | null = null;
let cachedEnvPasscodeSalt: Buffer | null = null;
let cachedEnvPasscodeHash: Buffer | null = null;
let fallbackMemoryPasscodeHash: string | null = null;
let workspaceTableReady = false;
let pgPool: Pool | null = null;

const unlockAttempts = new Map<string, { count: number; resetAt: number }>();

function getEnvPasscode(): string {
  return (process.env.ADMIN_PASSCODE || '').trim();
}

function getGithubPat(): string {
  return (process.env.GITHUB_PAT || '').trim();
}

function isPostgresConfigured(): boolean {
  return Boolean(process.env.DATABASE_URL);
}

function getPool(): Pool {
  if (!pgPool) {
    pgPool = new Pool({
      connectionString: process.env.DATABASE_URL,
      max: 5,
      idleTimeoutMillis: 30000,
    });
  }
  return pgPool;
}

function getOrCreateLocalSigningSecret(): string {
  try {
    if (fs.existsSync(WORKSPACE_SIGNING_KEY_FILE)) {
      const existing = fs.readFileSync(WORKSPACE_SIGNING_KEY_FILE, 'utf8').trim();
      if (existing.length >= 32) {
        return existing;
      }
    }
    const dir = path.dirname(WORKSPACE_SIGNING_KEY_FILE);
    fs.mkdirSync(dir, { recursive: true });
    const generated = crypto.randomBytes(32).toString('hex');
    fs.writeFileSync(WORKSPACE_SIGNING_KEY_FILE, `${generated}\n`, {
      encoding: 'utf8',
      mode: 0o600,
    });
    return generated;
  } catch {
    return crypto.randomBytes(32).toString('hex');
  }
}

function getSessionSigningKey(): Buffer {
  if (cachedSigningKey) {
    return cachedSigningKey;
  }
  const explicitSecret =
    process.env.ADMIN_SESSION_SECRET?.trim() ||
    process.env.SESSION_SIGNING_SECRET?.trim() ||
    process.env.DID_KEY_ENCRYPTION_SECRET?.trim();
  if (explicitSecret) {
    cachedSigningKey = crypto.scryptSync(
      explicitSecret,
      'exonovel-sync-session-explicit-v2',
      32
    );
    return cachedSigningKey;
  }

  const envPasscode = getEnvPasscode();
  if (envPasscode) {
    cachedSigningKey = crypto.scryptSync(
      `${envPasscode}\0${getGithubPat()}`,
      'exonovel-sync-session-v2',
      32
    );
    return cachedSigningKey;
  }

  const persistedSecret = getOrCreateLocalSigningSecret();
  cachedSigningKey = crypto.scryptSync(
    persistedSecret,
    'exonovel-sync-session-local-v2',
    32
  );
  return cachedSigningKey;
}

async function ensureWorkspacePasscodeTable(): Promise<void> {
  if (!isPostgresConfigured() || workspaceTableReady) return;
  const db = getPool();
  await db.query(`
    CREATE TABLE IF NOT EXISTS workspace_admin_passcode (
      id INTEGER PRIMARY KEY CHECK (id = 1),
      passcode_hash TEXT NOT NULL,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
  `);
  workspaceTableReady = true;
}

function hashPasscodeScrypt(passcode: string): string {
  const salt = crypto.randomBytes(16).toString('hex');
  const derived = crypto.scryptSync(passcode, salt, 64);
  return `${salt}:${derived.toString('hex')}`;
}

function verifyPasscodeScrypt(passcode: string, storedHash: string): boolean {
  const parts = storedHash.split(':');
  if (parts.length !== 2) return false;
  const [salt, keyHex] = parts;
  if (!salt || !keyHex) return false;
  const storedBuf = Buffer.from(keyHex, 'hex');
  if (storedBuf.length === 0) return false;
  const suppliedBuf = crypto.scryptSync(passcode, salt, storedBuf.length);
  return (
    storedBuf.length === suppliedBuf.length &&
    crypto.timingSafeEqual(storedBuf, suppliedBuf)
  );
}

function verifyEnvPasscodeScrypt(supplied: string, expected: string): boolean {
  if (
    cachedEnvPasscodeRaw !== expected ||
    !cachedEnvPasscodeSalt ||
    !cachedEnvPasscodeHash
  ) {
    cachedEnvPasscodeRaw = expected;
    cachedEnvPasscodeSalt = crypto.randomBytes(16);
    cachedEnvPasscodeHash = crypto.scryptSync(expected, cachedEnvPasscodeSalt, 32);
  }
  const candidate = crypto.scryptSync(supplied, cachedEnvPasscodeSalt, 32);
  return (
    candidate.length === cachedEnvPasscodeHash.length &&
    crypto.timingSafeEqual(candidate, cachedEnvPasscodeHash)
  );
}

function getClientAttemptKey(req?: Request): string {
  if (!req) return 'default';
  const forwarded = req.headers['x-forwarded-for'];
  const ip =
    typeof forwarded === 'string'
      ? forwarded.split(',')[0].trim()
      : req.ip || 'local';
  return ip || 'local';
}

function assertNotRateLimited(req?: Request): void {
  const key = getClientAttemptKey(req);
  const entry = unlockAttempts.get(key);
  if (!entry) return;
  if (Date.now() > entry.resetAt) {
    unlockAttempts.delete(key);
    return;
  }
  if (entry.count >= MAX_FAILED_UNLOCKS) {
    const secs = Math.max(1, Math.ceil((entry.resetAt - Date.now()) / 1000));
    throw new Error(`Too many incorrect passcode attempts. Try again in ${secs}s.`);
  }
}

function recordFailedAttempt(req?: Request): void {
  const key = getClientAttemptKey(req);
  const now = Date.now();
  const existing = unlockAttempts.get(key);
  if (!existing || now > existing.resetAt) {
    unlockAttempts.set(key, { count: 1, resetAt: now + UNLOCK_LOCKOUT_MS });
  } else {
    existing.count += 1;
  }
}

function clearFailedAttempts(req?: Request): void {
  unlockAttempts.delete(getClientAttemptKey(req));
}

async function getStoredPasscodeHash(): Promise<string | null> {
  if (isPostgresConfigured()) {
    try {
      await ensureWorkspacePasscodeTable();
      const db = getPool();
      const res = await db.query<{ passcode_hash: string }>(
        `SELECT passcode_hash FROM workspace_admin_passcode WHERE id = 1 LIMIT 1`
      );
      if (res.rows.length > 0) {
        return res.rows[0].passcode_hash;
      }
      return null;
    } catch (err) {
      console.error('Unable to read workspace passcode table:', err);
    }
  }
  return fallbackMemoryPasscodeHash;
}

export function isLoopbackHost(host: string): boolean {
  const lower = host.trim().toLowerCase();
  return (
    lower.startsWith('localhost:') ||
    lower === 'localhost' ||
    lower.startsWith('127.0.0.1:') ||
    lower === '127.0.0.1' ||
    lower.startsWith('[::1]:') ||
    lower === '[::1]'
  );
}

export function isAiStudioPreviewHost(host: string): boolean {
  const hostname = host.trim().toLowerCase().split(':')[0];
  return /^ais-(dev|pre)-[a-z0-9-]+(\.[a-z0-9-]+)*\.run\.app$/.test(hostname);
}

export function getAllowedAiStudioHosts(): Set<string> {
  const hosts = new Set<string>();
  const appUrl = process.env.APP_URL?.trim();
  if (appUrl) {
    try {
      const parsed = new URL(appUrl);
      const h = parsed.host.toLowerCase();
      hosts.add(h);
      if (h.startsWith('ais-dev-')) {
        hosts.add(h.replace(/^ais-dev-/, 'ais-pre-'));
      } else if (h.startsWith('ais-pre-')) {
        hosts.add(h.replace(/^ais-pre-/, 'ais-dev-'));
      }
    } catch {
      // Ignore malformed APP_URL
    }
  }
  return hosts;
}

export function isAiStudioRequestAllowed(req: Request): boolean {
  const rawHost = (req.headers['x-forwarded-host'] || req.headers.host || '')
    .toString()
    .split(',')[0]
    .trim()
    .toLowerCase();

  if (!rawHost) return false;
  if (isLoopbackHost(rawHost)) return true;

  const configuredHosts = getAllowedAiStudioHosts();
  if (configuredHosts.size > 0) {
    return configuredHosts.has(rawHost);
  }

  return isAiStudioPreviewHost(rawHost);
}

export function requireAiStudioEnvironment(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  if (isAiStudioRequestAllowed(req)) {
    next();
    return;
  }

  if (req.path.startsWith('/api/')) {
    res.status(403).json({
      error: 'Sync Console is restricted to AI Studio preview workspaces.',
      aiStudioRestricted: true,
    });
    return;
  }

  res
    .status(403)
    .type('html')
    .send(
      '<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Access Restricted</title></head><body style="font-family:system-ui,sans-serif;background:#111317;color:#eceef2;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0;padding:24px"><div style="max-width:440px;border:1px solid #262b36;border-radius:10px;padding:24px;background:#171a21"><h1 style="margin:0 0 8px;font-size:18px">AI Studio Access Only</h1><p style="margin:0;color:#9ca3af;font-size:14px;line-height:1.5">The ExoNovel Sync Console runs exclusively inside AI Studio preview workspaces and cannot be accessed from external public hosts.</p></div></body></html>'
    );
}

export function issueWorkspaceSessionToken(): string {
  const payload = {
    sub: 'workspace-admin',
    iat: Date.now(),
    exp: Date.now() + SESSION_MAX_AGE_SECONDS * 1000,
    nonce: crypto.randomBytes(8).toString('hex'),
  };
  const encoded = Buffer.from(JSON.stringify(payload), 'utf8').toString('base64url');
  const sig = crypto
    .createHmac('sha256', getSessionSigningKey())
    .update(encoded)
    .digest('base64url');
  return `ws.${encoded}.${sig}`;
}

export function verifyWorkspaceSessionToken(token: string): boolean {
  if (!token.startsWith('ws.')) return false;
  const parts = token.split('.');
  if (parts.length !== 3) return false;
  const [, encoded, sig] = parts;
  if (!encoded || !sig) return false;

  const expectedSig = crypto
    .createHmac('sha256', getSessionSigningKey())
    .update(encoded)
    .digest('base64url');

  const sigBuf = Buffer.from(sig, 'utf8');
  const expectedBuf = Buffer.from(expectedSig, 'utf8');
  if (sigBuf.length !== expectedBuf.length) return false;
  if (!crypto.timingSafeEqual(sigBuf, expectedBuf)) return false;

  try {
    const parsed = JSON.parse(
      Buffer.from(encoded, 'base64url').toString('utf8')
    ) as { sub?: string; exp?: number };
    if (parsed.sub !== 'workspace-admin') return false;
    if (typeof parsed.exp !== 'number' || Date.now() > parsed.exp) return false;
    return true;
  } catch {
    return false;
  }
}

export function extractWorkspaceCookieToken(req: Request): string {
  const rawCookie = req.headers.cookie || '';
  if (!rawCookie) return '';
  for (const part of rawCookie.split(';')) {
    const idx = part.indexOf('=');
    if (idx === -1) continue;
    const name = part.slice(0, idx).trim();
    if (name === SESSION_COOKIE_NAME) {
      try {
        return decodeURIComponent(part.slice(idx + 1).trim());
      } catch {
        return part.slice(idx + 1).trim();
      }
    }
  }
  return '';
}

export function setWorkspaceSessionCookie(
  req: Request,
  res: Response,
  token: string
): void {
  const isHttps =
    req.secure ||
    req.headers['x-forwarded-proto'] === 'https' ||
    Boolean(process.env.APP_URL?.startsWith('https://'));
  const attrs = [
    `${SESSION_COOKIE_NAME}=${encodeURIComponent(token)}`,
    'Path=/',
    `Max-Age=${SESSION_MAX_AGE_SECONDS}`,
    'HttpOnly',
  ];
  if (isHttps) {
    attrs.push('Secure', 'SameSite=None', 'Partitioned');
  } else {
    attrs.push('SameSite=Lax');
  }
  res.setHeader('Set-Cookie', attrs.join('; '));
}

export function clearWorkspaceSessionCookie(req: Request, res: Response): void {
  const isHttps =
    req.secure ||
    req.headers['x-forwarded-proto'] === 'https' ||
    Boolean(process.env.APP_URL?.startsWith('https://'));
  const attrs = [
    `${SESSION_COOKIE_NAME}=`,
    'Path=/',
    'Max-Age=0',
    'HttpOnly',
  ];
  if (isHttps) {
    attrs.push('Secure', 'SameSite=None', 'Partitioned');
  } else {
    attrs.push('SameSite=Lax');
  }
  res.setHeader('Set-Cookie', attrs.join('; '));
}

function getRequestAllowedOrigins(req: Request): Set<string> {
  const hosts = getAllowedAiStudioHosts();
  const addHost = (val: string | undefined) => {
    if (!val) return;
    for (const part of val.split(',')) {
      const h = part.trim().toLowerCase();
      if (h) hosts.add(h);
    }
  };
  addHost(
    typeof req.headers['x-forwarded-host'] === 'string'
      ? req.headers['x-forwarded-host']
      : undefined
  );
  addHost(typeof req.headers.host === 'string' ? req.headers.host : undefined);
  return hosts;
}

export function hasValidCsrfGuard(req: Request): boolean {
  const requestedWith = req.headers['x-requested-with'];
  if (requestedWith !== SYNC_REQUESTED_WITH && requestedWith !== 'XMLHttpRequest') {
    return false;
  }
  const origin = req.headers.origin;
  if (!origin) {
    return isAiStudioRequestAllowed(req);
  }
  try {
    const originHost = new URL(origin).host.toLowerCase();
    return getRequestAllowedOrigins(req).has(originHost);
  } catch {
    return false;
  }
}

export async function getWorkspaceAuthStatus(req?: Request): Promise<{
  claimed: boolean;
  source: 'env' | 'claimed' | 'unclaimed';
  unlocked: boolean;
}> {
  const bearer = req?.headers.authorization?.startsWith('Bearer ')
    ? req.headers.authorization.slice(7).trim()
    : '';
  const cookieToken = req ? extractWorkspaceCookieToken(req) : '';
  const unlocked = Boolean(
    (bearer && verifyWorkspaceSessionToken(bearer)) ||
      (cookieToken && verifyWorkspaceSessionToken(cookieToken))
  );

  if (getEnvPasscode()) {
    return { claimed: true, source: 'env', unlocked };
  }
  const stored = await getStoredPasscodeHash();
  if (stored) {
    return { claimed: true, source: 'claimed', unlocked };
  }
  return { claimed: false, source: 'unclaimed', unlocked };
}

export async function claimWorkspacePasscode(passcode: string): Promise<string> {
  const cleaned = passcode.trim();
  if (cleaned.length < 4 || cleaned.length > 128) {
    throw new Error('Passcode must be between 4 and 128 characters.');
  }
  if (getEnvPasscode()) {
    throw new Error('Passcode is already configured via ADMIN_PASSCODE.');
  }

  const hashed = hashPasscodeScrypt(cleaned);
  if (isPostgresConfigured()) {
    await ensureWorkspacePasscodeTable();
    const db = getPool();
    const inserted = await db.query<{ id: number }>(
      `INSERT INTO workspace_admin_passcode (id, passcode_hash)
       VALUES (1, $1)
       ON CONFLICT (id) DO NOTHING
       RETURNING id`,
      [hashed]
    );
    if (inserted.rows.length === 0) {
      throw new Error('Workspace passcode is already set. Use unlock instead.');
    }
  } else {
    if (fallbackMemoryPasscodeHash) {
      throw new Error('Workspace passcode is already set. Use unlock instead.');
    }
    fallbackMemoryPasscodeHash = hashed;
  }

  return issueWorkspaceSessionToken();
}

export async function unlockWithWorkspacePasscode(
  passcode: string,
  req?: Request
): Promise<string> {
  assertNotRateLimited(req);
  const cleaned = passcode.trim();
  if (!cleaned) {
    throw new Error('Passcode is required.');
  }

  const envPass = getEnvPasscode();
  if (envPass) {
    if (!verifyEnvPasscodeScrypt(cleaned, envPass)) {
      recordFailedAttempt(req);
      throw new Error('Invalid passcode.');
    }
    clearFailedAttempts(req);
    return issueWorkspaceSessionToken();
  }

  const stored = await getStoredPasscodeHash();
  if (!stored) {
    throw new Error('No passcode has been set yet.');
  }
  if (!verifyPasscodeScrypt(cleaned, stored)) {
    recordFailedAttempt(req);
    throw new Error('Invalid passcode.');
  }

  clearFailedAttempts(req);
  return issueWorkspaceSessionToken();
}

export function requireWorkspaceSession(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  if (!isAiStudioRequestAllowed(req)) {
    res.status(403).json({
      error: 'Sync Console is restricted to AI Studio preview workspaces.',
      aiStudioRestricted: true,
    });
    return;
  }

  const method = req.method.toUpperCase();
  const isMutation = method !== 'GET' && method !== 'HEAD' && method !== 'OPTIONS';

  const authHeader = req.headers.authorization || '';
  const bearerToken = authHeader.startsWith('Bearer ')
    ? authHeader.slice(7).trim()
    : '';
  const cookieToken = extractWorkspaceCookieToken(req);

  const hasValidToken =
    (bearerToken && verifyWorkspaceSessionToken(bearerToken)) ||
    (cookieToken && verifyWorkspaceSessionToken(cookieToken));

  if (!hasValidToken) {
    res.status(401).json({ error: 'Workspace unlock required.' });
    return;
  }

  if (isMutation && !hasValidCsrfGuard(req)) {
    res.status(403).json({ error: 'Invalid request origin or missing sync header.' });
    return;
  }

  next();
}
