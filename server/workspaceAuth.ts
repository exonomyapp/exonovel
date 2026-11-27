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
let cachedSigningKey: Buffer | null = null;
const SESSION_COOKIE_NAME = 'exonovel_ws_session';
const SESSION_MAX_AGE_SECONDS = 60 * 60 * 24 * 7; // 7 days
let fallbackMemoryPasscodeHash: string | null = null;
let workspaceTableReady = false;
let pgPool: Pool | null = null;

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
    process.env.DID_KEY_ENCRYPTION_SECRET?.trim();
  if (explicitSecret) {
    cachedSigningKey = crypto
      .createHash('sha256')
      .update(`exonovel-ws-session:${explicitSecret}`)
      .digest();
    return cachedSigningKey;
  }

  const envPasscode = getEnvPasscode();
  if (envPasscode) {
    cachedSigningKey = crypto
      .createHmac('sha256', envPasscode)
      .update('exonovel-ws-session-v1')
      .digest();
    return cachedSigningKey;
  }

  const persistedSecret = getOrCreateLocalSigningSecret();
  cachedSigningKey = crypto
    .createHash('sha256')
    .update(`exonovel-ws-session:${persistedSecret}`)
    .digest();
  return cachedSigningKey;
}

function getEnvPasscode(): string {
  return (process.env.ADMIN_PASSCODE || '').trim();
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

function verifyEnvPasscode(supplied: string, expected: string): boolean {
  const a = crypto.createHash('sha256').update(supplied, 'utf8').digest();
  const b = crypto.createHash('sha256').update(expected, 'utf8').digest();
  return crypto.timingSafeEqual(a, b);
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

export function setWorkspaceSessionCookie(res: Response, token: string): void {
  res.setHeader(
    'Set-Cookie',
    `${SESSION_COOKIE_NAME}=${encodeURIComponent(token)}; Path=/; Max-Age=${SESSION_MAX_AGE_SECONDS}; HttpOnly; Secure; SameSite=None; Partitioned`
  );
}

export function clearWorkspaceSessionCookie(res: Response): void {
  res.setHeader(
    'Set-Cookie',
    `${SESSION_COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; Secure; SameSite=None; Partitioned`
  );
}

export function hasValidCsrfGuard(req: Request): boolean {
  if (req.headers['x-requested-with'] === 'XMLHttpRequest') {
    return true;
  }
  const origin = req.headers.origin;
  const host = (req.headers['x-forwarded-host'] || req.headers.host || '')
    .toString()
    .split(',')[0]
    .trim();
  if (origin && host) {
    try {
      return new URL(origin).host === host;
    } catch {
      return false;
    }
  }
  return false;
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

export async function unlockWithWorkspacePasscode(passcode: string): Promise<string> {
  const cleaned = passcode.trim();
  if (!cleaned) {
    throw new Error('Passcode is required.');
  }

  const envPass = getEnvPasscode();
  if (envPass) {
    if (!verifyEnvPasscode(cleaned, envPass)) {
      throw new Error('Invalid passcode.');
    }
    return issueWorkspaceSessionToken();
  }

  const stored = await getStoredPasscodeHash();
  if (!stored) {
    throw new Error('No passcode has been set yet.');
  }
  if (!verifyPasscodeScrypt(cleaned, stored)) {
    throw new Error('Invalid passcode.');
  }

  return issueWorkspaceSessionToken();
}

export function requireWorkspaceSession(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const authHeader = req.headers.authorization || '';
  const bearerToken = authHeader.startsWith('Bearer ')
    ? authHeader.slice(7).trim()
    : '';

  if (bearerToken && verifyWorkspaceSessionToken(bearerToken)) {
    next();
    return;
  }

  const cookieToken = extractWorkspaceCookieToken(req);
  if (cookieToken && verifyWorkspaceSessionToken(cookieToken)) {
    if (req.method !== 'GET' && req.method !== 'HEAD' && !hasValidCsrfGuard(req)) {
      res.status(403).json({ error: 'Missing request origin verification.' });
      return;
    }
    next();
    return;
  }

  res.status(401).json({ error: 'Workspace unlock required.' });
}
