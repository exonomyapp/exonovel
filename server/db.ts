import { Pool } from 'pg';
import crypto from 'node:crypto';

let pool: Pool | null = null;
let schemaInitialized = false;

export function getAdminEmail(): string {
  return (process.env.ADMIN_EMAIL || 'exonomyapp@gmail.com').trim().toLowerCase();
}

export function isPostgresConfigured(): boolean {
  return Boolean(process.env.DATABASE_URL);
}

export function getPool(): Pool {
  if (!process.env.DATABASE_URL) {
    throw new Error('DATABASE_URL is not configured.');
  }
  if (!pool) {
    pool = new Pool({
      connectionString: process.env.DATABASE_URL,
      max: 10,
      idleTimeoutMillis: 30000,
    });
  }
  return pool;
}

export async function ensureSchema(): Promise<void> {
  if (!isPostgresConfigured() || schemaInitialized) return;
  const db = getPool();

  await db.query(`
    CREATE TABLE IF NOT EXISTS user_identities (
      id SERIAL PRIMARY KEY,
      did_peer TEXT NOT NULL UNIQUE,
      public_key TEXT NOT NULL,
      encrypted_private_key TEXT NOT NULL,
      auth_user_id TEXT UNIQUE,
      email TEXT,
      provider TEXT NOT NULL DEFAULT 'better-auth',
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
  `);

  await db.query(`
    CREATE TABLE IF NOT EXISTS docs_access_allowlist (
      email TEXT PRIMARY KEY,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      role TEXT NOT NULL DEFAULT 'reader',
      note TEXT NOT NULL DEFAULT '',
      added_by TEXT NOT NULL DEFAULT 'system',
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
  `);

  await db.query(`
    CREATE TABLE IF NOT EXISTS docs_storage_snapshots (
      id SERIAL PRIMARY KEY,
      doc_path TEXT NOT NULL,
      object_key TEXT NOT NULL UNIQUE,
      size_bytes INTEGER NOT NULL DEFAULT 0,
      uploaded_by_did TEXT NOT NULL,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
  `);

  const adminEmail = getAdminEmail();
  await db.query(
    `
      INSERT INTO docs_access_allowlist (email, enabled, role, note, added_by)
      VALUES ($1, TRUE, 'admin', 'Primary ExoNovel documentation owner', 'system')
      ON CONFLICT (email) DO UPDATE
      SET enabled = TRUE, role = 'admin', updated_at = NOW();
    `,
    [adminEmail]
  );

  schemaInitialized = true;
}

function getKeyWrappingSecret(): Buffer {
  const raw =
    process.env.DID_KEY_ENCRYPTION_SECRET ||
    process.env.NEON_AUTH_COOKIE_SECRET ||
    process.env.DATABASE_URL ||
    'exonovel-dev-wrapping-fallback';
  return crypto.createHash('sha256').update(raw).digest();
}

function encryptPrivateKey(pem: string): string {
  const key = getKeyWrappingSecret();
  const iv = crypto.randomBytes(12);
  const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
  const encrypted = Buffer.concat([cipher.update(pem, 'utf8'), cipher.final()]);
  const tag = cipher.getAuthTag();
  return `${iv.toString('base64')}.${tag.toString('base64')}.${encrypted.toString('base64')}`;
}

function base58btcEncode(bytes: Uint8Array): string {
  const ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  let num = BigInt('0x' + Buffer.from(bytes).toString('hex'));
  let encoded = '';
  while (num > 0n) {
    const mod = Number(num % 58n);
    encoded = ALPHABET[mod] + encoded;
    num = num / 58n;
  }
  for (const b of bytes) {
    if (b === 0) encoded = '1' + encoded;
    else break;
  }
  return encoded;
}

function mintDidPeerKeypair(): {
  didPeer: string;
  publicKeyB58: string;
  encryptedPrivateKey: string;
} {
  const { publicKey, privateKey } = crypto.generateKeyPairSync('ed25519');
  const spkiDer = publicKey.export({ format: 'der', type: 'spki' });
  const rawPublicKey = spkiDer.subarray(spkiDer.length - 32);
  // Multicodec ed25519-pub prefix (0xed, 0x01) + 32-byte key -> z-prefixed base58btc -> did:peer:0z...
  const multicodecKey = Buffer.concat([Buffer.from([0xed, 0x01]), rawPublicKey]);
  const multibase = 'z' + base58btcEncode(multicodecKey);
  const didPeer = `did:peer:0${multibase}`;

  const pkcs8Pem = privateKey.export({ format: 'pem', type: 'pkcs8' }).toString();
  return {
    didPeer,
    publicKeyB58: multibase,
    encryptedPrivateKey: encryptPrivateKey(pkcs8Pem),
  };
}

export interface IdentityRecord {
  didPeer: string;
  email: string | null;
  authUserId: string;
  provider: string;
  createdAt: string;
}

export async function resolveOrMintDidPeer(params: {
  authUserId: string;
  email: string | null;
  provider?: string;
}): Promise<IdentityRecord> {
  await ensureSchema();
  const db = getPool();

  const existing = await db.query<{
    did_peer: string;
    email: string | null;
    auth_user_id: string;
    provider: string;
    created_at: Date;
  }>(
    `SELECT did_peer, email, auth_user_id, provider, created_at
     FROM user_identities
     WHERE auth_user_id = $1
     LIMIT 1`,
    [params.authUserId]
  );

  if (existing.rows.length > 0) {
    const row = existing.rows[0];
    if (params.email && row.email !== params.email) {
      await db.query(
        `UPDATE user_identities SET email = $1, updated_at = NOW() WHERE auth_user_id = $2`,
        [params.email, params.authUserId]
      );
    }
    return {
      didPeer: row.did_peer,
      email: params.email ?? row.email,
      authUserId: row.auth_user_id,
      provider: row.provider,
      createdAt: row.created_at.toISOString(),
    };
  }

  const minted = mintDidPeerKeypair();
  const inserted = await db.query<{
    did_peer: string;
    email: string | null;
    auth_user_id: string;
    provider: string;
    created_at: Date;
  }>(
    `INSERT INTO user_identities (did_peer, public_key, encrypted_private_key, auth_user_id, email, provider)
     VALUES ($1, $2, $3, $4, $5, $6)
     ON CONFLICT (auth_user_id) DO UPDATE
     SET email = COALESCE(EXCLUDED.email, user_identities.email), updated_at = NOW()
     RETURNING did_peer, email, auth_user_id, provider, created_at`,
    [
      minted.didPeer,
      minted.publicKeyB58,
      minted.encryptedPrivateKey,
      params.authUserId,
      params.email,
      params.provider || 'better-auth',
    ]
  );

  const row = inserted.rows[0];
  return {
    didPeer: row.did_peer,
    email: row.email,
    authUserId: row.auth_user_id,
    provider: row.provider,
    createdAt: row.created_at.toISOString(),
  };
}

export interface AllowlistEntry {
  email: string;
  enabled: boolean;
  role: string;
  note: string;
  addedBy: string;
  updatedAt: string;
}

export async function isEmailAuthorizedForDocs(email: string | null): Promise<{
  authorized: boolean;
  isAdmin: boolean;
}> {
  if (!email) return { authorized: false, isAdmin: false };
  const normalized = email.trim().toLowerCase();
  const adminEmail = getAdminEmail();

  if (normalized === adminEmail) {
    if (isPostgresConfigured()) {
      await ensureSchema();
    }
    return { authorized: true, isAdmin: true };
  }

  if (!isPostgresConfigured()) {
    return { authorized: false, isAdmin: false };
  }

  await ensureSchema();
  const db = getPool();
  const res = await db.query<{ enabled: boolean; role: string }>(
    `SELECT enabled, role FROM docs_access_allowlist WHERE email = $1 LIMIT 1`,
    [normalized]
  );

  if (res.rows.length === 0) {
    return { authorized: false, isAdmin: false };
  }

  const row = res.rows[0];
  return {
    authorized: Boolean(row.enabled),
    isAdmin: row.enabled && row.role === 'admin',
  };
}

export async function listAllowlist(): Promise<AllowlistEntry[]> {
  await ensureSchema();
  const db = getPool();
  const res = await db.query<{
    email: string;
    enabled: boolean;
    role: string;
    note: string;
    added_by: string;
    updated_at: Date;
  }>(
    `SELECT email, enabled, role, note, added_by, updated_at
     FROM docs_access_allowlist
     ORDER BY CASE WHEN role = 'admin' THEN 0 ELSE 1 END, email ASC`
  );

  return res.rows.map((r) => ({
    email: r.email,
    enabled: r.enabled,
    role: r.role,
    note: r.note,
    addedBy: r.added_by,
    updatedAt: r.updated_at.toISOString(),
  }));
}

export async function upsertAllowlistEntry(params: {
  email: string;
  enabled: boolean;
  note?: string;
  addedBy: string;
}): Promise<AllowlistEntry> {
  await ensureSchema();
  const db = getPool();
  const normalized = params.email.trim().toLowerCase();
  const adminEmail = getAdminEmail();
  const role = normalized === adminEmail ? 'admin' : 'reader';
  const enabled = normalized === adminEmail ? true : params.enabled;

  const res = await db.query<{
    email: string;
    enabled: boolean;
    role: string;
    note: string;
    added_by: string;
    updated_at: Date;
  }>(
    `INSERT INTO docs_access_allowlist (email, enabled, role, note, added_by, updated_at)
     VALUES ($1, $2, $3, $4, $5, NOW())
     ON CONFLICT (email) DO UPDATE
     SET enabled = $2,
         note = COALESCE(NULLIF($4, ''), docs_access_allowlist.note),
         updated_at = NOW()
     RETURNING email, enabled, role, note, added_by, updated_at`,
    [normalized, enabled, role, params.note?.trim() ?? '', params.addedBy]
  );

  const r = res.rows[0];
  return {
    email: r.email,
    enabled: r.enabled,
    role: r.role,
    note: r.note,
    addedBy: r.added_by,
    updatedAt: r.updated_at.toISOString(),
  };
}

export async function deleteAllowlistEntry(email: string): Promise<boolean> {
  await ensureSchema();
  const normalized = email.trim().toLowerCase();
  if (normalized === getAdminEmail()) {
    throw new Error('Cannot remove primary admin account.');
  }
  const db = getPool();
  const res = await db.query(
    `DELETE FROM docs_access_allowlist WHERE email = $1`,
    [normalized]
  );
  return (res.rowCount ?? 0) > 0;
}

export async function listIdentities(): Promise<IdentityRecord[]> {
  await ensureSchema();
  const db = getPool();
  const res = await db.query<{
    did_peer: string;
    email: string | null;
    auth_user_id: string;
    provider: string;
    created_at: Date;
  }>(
    `SELECT did_peer, email, auth_user_id, provider, created_at
     FROM user_identities
     ORDER BY created_at DESC
     LIMIT 100`
  );

  return res.rows.map((r) => ({
    didPeer: r.did_peer,
    email: r.email,
    authUserId: r.auth_user_id,
    provider: r.provider,
    createdAt: r.created_at.toISOString(),
  }));
}

export async function recordSnapshot(params: {
  docPath: string;
  objectKey: string;
  sizeBytes: number;
  uploadedByDid: string;
}): Promise<void> {
  await ensureSchema();
  const db = getPool();
  await db.query(
    `INSERT INTO docs_storage_snapshots (doc_path, object_key, size_bytes, uploaded_by_did)
     VALUES ($1, $2, $3, $4)`,
    [params.docPath, params.objectKey, params.sizeBytes, params.uploadedByDid]
  );
}

export async function listSnapshots(): Promise<
  Array<{
    id: number;
    docPath: string;
    objectKey: string;
    sizeBytes: number;
    uploadedByDid: string;
    createdAt: string;
  }>
> {
  await ensureSchema();
  const db = getPool();
  const res = await db.query<{
    id: number;
    doc_path: string;
    object_key: string;
    size_bytes: number;
    uploaded_by_did: string;
    created_at: Date;
  }>(
    `SELECT id, doc_path, object_key, size_bytes, uploaded_by_did, created_at
     FROM docs_storage_snapshots
     ORDER BY created_at DESC
     LIMIT 50`
  );

  return res.rows.map((r) => ({
    id: r.id,
    docPath: r.doc_path,
    objectKey: r.object_key,
    sizeBytes: r.size_bytes,
    uploadedByDid: r.uploaded_by_did,
    createdAt: r.created_at.toISOString(),
  }));
}
