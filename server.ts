import { execFile, execFileSync } from 'node:child_process';
import crypto from 'node:crypto';
import dotenv from 'dotenv';
import express from 'express';
import fs from 'node:fs';
import path from 'node:path';
import { createServer as createViteServer } from 'vite';
import {
  isAuthConfigured,
  requireAdminAccess,
  requireAuthenticatedUser,
  requireDocsAccess,
  type AuthenticatedRequest,
} from './server/auth.ts';
import {
  deleteAllowlistEntry,
  ensureSchema,
  getAdminEmail,
  getPool,
  isPostgresConfigured,
  listAllowlist,
  listIdentities,
  listSnapshots,
  recordSnapshot,
  upsertAllowlistEntry,
} from './server/db.ts';
import { extractTitleAndHeadings, getDocsTree, resolveSafeDocPath } from './server/docs.ts';
import {
  getSnapshotPresignedUrl,
  isStorageConfigured,
  uploadDocSnapshot,
} from './server/storage.ts';

const ROOT_DIR = process.cwd();
const envLocalPath = path.join(ROOT_DIR, '.env.local');
const envPath = path.join(ROOT_DIR, '.env');
if (fs.existsSync(envLocalPath)) {
  dotenv.config({ path: envLocalPath });
}
if (fs.existsSync(envPath)) {
  dotenv.config({ path: envPath });
}

const PORT = Number(process.env.PORT || 3000);
let neonDeployStatus: { attempted: boolean; ok: boolean; message: string } = {
  attempted: false,
  ok: false,
  message: 'Not needed or not run',
};

const NEON_ORG_ID = process.env.NEON_ORG_ID || 'org-still-credit-09220145';
const NEON_PROJECT_ID = process.env.NEON_PROJECT_ID || 'curly-pine-66858617';
const NEON_BRANCH = process.env.NEON_BRANCH || 'production';

function resolveNeonBin(): string {
  const localBin = path.join(ROOT_DIR, 'node_modules', '.bin', 'neon');
  return fs.existsSync(localBin) ? localBin : 'neon';
}

function bootstrapNeonDeployIfNeeded() {
  const needsNeonVars =
    !process.env.DATABASE_URL ||
    !process.env.NEON_AUTH_BASE_URL ||
    !process.env.AWS_ENDPOINT_URL_S3;
  if (!process.env.NEON_API_KEY || !needsNeonVars) {
    return;
  }

  const neonBin = resolveNeonBin();
  const neonContextFile = path.join(ROOT_DIR, '.neon');
  try {
    if (!fs.existsSync(neonContextFile)) {
      try {
        execFileSync(
          neonBin,
          ['set-context', '--org-id', NEON_ORG_ID, '--project-id', NEON_PROJECT_ID],
          { cwd: ROOT_DIR, env: process.env, encoding: 'utf-8', timeout: 20000 }
        );
      } catch {
        // Proceed with explicit --project-id and --branch flags
      }
    }

    let out = '';
    try {
      out = execFileSync(
        neonBin,
        ['env', 'pull', '--project-id', NEON_PROJECT_ID, '--branch', NEON_BRANCH, '--file', '.env.local'],
        {
          cwd: ROOT_DIR,
          env: process.env,
          encoding: 'utf-8',
          timeout: 60000,
        }
      );
    } catch {
      out = execFileSync(
        neonBin,
        ['deploy', '--project-id', NEON_PROJECT_ID, '--branch', NEON_BRANCH, '-y'],
        {
          cwd: ROOT_DIR,
          env: process.env,
          encoding: 'utf-8',
          timeout: 90000,
        }
      );
    }

    if (fs.existsSync(envLocalPath)) {
      dotenv.config({ path: envLocalPath, override: true });
    }
    neonDeployStatus = {
      attempted: true,
      ok: Boolean(process.env.DATABASE_URL && process.env.NEON_AUTH_BASE_URL),
      message: out.trim() || 'Pulled Neon environment configuration',
    };
    console.log('Neon environment sync completed successfully.');
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    neonDeployStatus = {
      attempted: true,
      ok: false,
      message: msg,
    };
    console.error('Neon bootstrap failed:', msg);
  }
}

let neonOAuthSyncStatus: {
  googleConfigured: boolean;
  googleIsCustom: boolean;
  trustedDomains: string[];
  lastError: string | null;
} = {
  googleConfigured: false,
  googleIsCustom: false,
  trustedDomains: [],
  lastError: null,
};

function syncTrustedOriginsInBackground() {
  if (!process.env.NEON_API_KEY) return;
  const origins = new Set<string>(['https://exonovel.exosystems.net']);
  if (process.env.PRODUCTION_APP_URL) {
    try {
      origins.add(new URL(process.env.PRODUCTION_APP_URL).origin);
    } catch {
      // Ignore malformed PRODUCTION_APP_URL
    }
  }
  if (process.env.APP_URL) {
    try {
      const devOrigin = new URL(process.env.APP_URL).origin;
      origins.add(devOrigin);
      if (devOrigin.includes('https://ais-dev-')) {
        origins.add(devOrigin.replace('https://ais-dev-', 'https://ais-pre-'));
      } else if (devOrigin.includes('https://ais-pre-')) {
        origins.add(devOrigin.replace('https://ais-pre-', 'https://ais-dev-'));
      }
    } catch {
      // Ignore malformed APP_URL
    }
  }

  for (const origin of origins) {
    execFile(
      resolveNeonBin(),
      ['neon-auth', 'domain', 'add', origin],
      { cwd: ROOT_DIR, env: process.env, timeout: 20000 },
      () => {
        execFile(
          resolveNeonBin(),
          ['neon-auth', 'domain', 'list', '-o', 'json'],
          { cwd: ROOT_DIR, env: process.env, timeout: 20000 },
          (listErr, stdout) => {
            if (!listErr && stdout) {
              try {
                const parsed = JSON.parse(stdout) as Array<{ domain?: string } | string>;
                neonOAuthSyncStatus.trustedDomains = parsed
                  .map((item) => (typeof item === 'string' ? item : item.domain || ''))
                  .filter(Boolean);
              } catch {
                // Ignore non-JSON output
              }
            }
          }
        );
      }
    );
  }
}

function refreshOAuthProviderStatus() {
  if (!process.env.NEON_API_KEY) return;
  execFile(
    resolveNeonBin(),
    ['neon-auth', 'oauth-provider', 'list', '-o', 'json'],
    { cwd: ROOT_DIR, env: process.env, timeout: 20000 },
    (err, stdout) => {
      if (err || !stdout) return;
      try {
        const list = JSON.parse(stdout) as Array<{
          id?: string;
          provider_id?: string;
          type?: string;
          client_id?: string;
        }>;
        const google = list.find(
          (p) => p.id === 'google' || p.provider_id === 'google'
        );
        if (google) {
          neonOAuthSyncStatus.googleConfigured = true;
          neonOAuthSyncStatus.googleIsCustom =
            google.type === 'custom' || Boolean(google.client_id);
        }
      } catch {
        // Ignore parse error
      }
    }
  );
}

function syncOAuthProvider(
  providerId: 'google' | 'github',
  clientId: string,
  clientSecret: string
) {
  const args = (verb: 'update' | 'add') => [
    'neon-auth',
    'oauth-provider',
    verb,
    '--provider-id',
    providerId,
    '--oauth-client-id',
    clientId,
    '--oauth-client-secret',
    clientSecret,
  ];

  execFile(
    resolveNeonBin(),
    args('update'),
    { cwd: ROOT_DIR, env: process.env, timeout: 25000 },
    (updateErr, _out, updateStderr) => {
      if (!updateErr) {
        console.log(`Synced custom ${providerId} OAuth credentials to Neon Auth.`);
        neonOAuthSyncStatus.lastError = null;
        refreshOAuthProviderStatus();
        return;
      }
      execFile(
        resolveNeonBin(),
        args('add'),
        { cwd: ROOT_DIR, env: process.env, timeout: 25000 },
        (addErr, _addOut, addStderr) => {
          if (!addErr) {
            console.log(`Added custom ${providerId} OAuth credentials to Neon Auth.`);
            neonOAuthSyncStatus.lastError = null;
            refreshOAuthProviderStatus();
          } else {
            const safeMsg = (addStderr || updateStderr || addErr.message || '').slice(0, 200);
            neonOAuthSyncStatus.lastError = safeMsg;
            console.error(`Unable to sync ${providerId} OAuth credentials:`, safeMsg);
            refreshOAuthProviderStatus();
          }
        }
      );
    }
  );
}

function syncOAuthProvidersInBackground() {
  if (!process.env.NEON_API_KEY) return;
  const googleId = process.env.GOOGLE_OAUTH_CLIENT_ID?.trim();
  const googleSecret = process.env.GOOGLE_OAUTH_CLIENT_SECRET?.trim();
  if (googleId && googleSecret) {
    syncOAuthProvider('google', googleId, googleSecret);
  } else {
    refreshOAuthProviderStatus();
  }

  const githubId = process.env.GITHUB_OAUTH_CLIENT_ID?.trim();
  const githubSecret = process.env.GITHUB_OAUTH_CLIENT_SECRET?.trim();
  if (githubId && githubSecret) {
    syncOAuthProvider('github', githubId, githubSecret);
  }
}

let neonEmailSyncStatus: {
  detectedKeys: string[];
  brevoApiAuthorized: boolean | null;
  emailProviderType: string | null;
  host: string | null;
  port: number | null;
  senderEmail: string | null;
  synced: boolean;
  testResult: string | null;
  lastError: string | null;
} = {
  detectedKeys: [],
  brevoApiAuthorized: null,
  emailProviderType: null,
  host: null,
  port: null,
  senderEmail: null,
  synced: false,
  testResult: null,
  lastError: null,
};

function getEmailProviderConfig(): Promise<{
  type?: string;
  host?: string;
  port?: number;
  sender_email?: string;
} | null> {
  return new Promise((resolve) => {
    if (!process.env.NEON_API_KEY) {
      resolve(null);
      return;
    }
    execFile(
      resolveNeonBin(),
      ['neon-auth', 'config', 'email-provider', 'get', '-o', 'json'],
      { cwd: ROOT_DIR, env: process.env, timeout: 20000 },
      (err, stdout) => {
        if (err || !stdout) {
          resolve(null);
          return;
        }
        try {
          const cfg = JSON.parse(stdout) as {
            type?: string;
            host?: string;
            port?: number;
            sender_email?: string;
          };
          neonEmailSyncStatus.emailProviderType = cfg.type ?? null;
          neonEmailSyncStatus.host = cfg.host ?? null;
          neonEmailSyncStatus.port = typeof cfg.port === 'number' ? cfg.port : null;
          neonEmailSyncStatus.senderEmail = cfg.sender_email ?? null;
          resolve(cfg);
        } catch {
          resolve(null);
        }
      }
    );
  });
}

async function resolveBrevoConfigFromEnvAndApi() {
  const keys = Object.keys(process.env)
    .filter((k) => /(brevo|smtp)/i.test(k) && Boolean(process.env[k]?.trim()))
    .sort();
  neonEmailSyncStatus.detectedKeys = keys;

  const pick = (...candidates: string[]) => {
    for (const c of candidates) {
      const val = process.env[c]?.trim();
      if (val) return val;
    }
    return '';
  };

  let host = pick('BREVO_SMTP_HOST', 'BREVO_HOST', 'SMTP_HOST');
  let port = pick('BREVO_SMTP_PORT', 'BREVO_PORT', 'SMTP_PORT');
  let username = pick(
    'BREVO_SMTP_LOGIN',
    'BREVO_SMTP_USER',
    'BREVO_SMTP_USERNAME',
    'BREVO_LOGIN',
    'BREVO_USER',
    'BREVO_USERNAME',
    'SMTP_LOGIN',
    'SMTP_USER',
    'SMTP_USERNAME'
  );
  let password = pick(
    'BREVO_SMTP_KEY',
    'BREVO_SMTP_PASSWORD',
    'BREVO_SMTP_PASS',
    'BREVO_SMTP_SECRET',
    'BREVO_KEY',
    'BREVO_PASSWORD',
    'BREVO_PASS',
    'SMTP_KEY',
    'SMTP_PASSWORD',
    'SMTP_PASS'
  );
  let senderEmail = pick(
    'BREVO_SENDER_EMAIL',
    'BREVO_FROM_EMAIL',
    'BREVO_SENDER',
    'BREVO_FROM',
    'BREVO_EMAIL',
    'SMTP_SENDER_EMAIL',
    'SMTP_FROM_EMAIL',
    'SMTP_SENDER',
    'SMTP_FROM',
    'MAIL_FROM',
    'SENDER_EMAIL',
    'FROM_EMAIL'
  );
  let senderName = 'ExoNovel';

  for (const key of keys) {
    const val = process.env[key]!.trim();
    const upper = key.toUpperCase();
    if (upper === 'BREVO_API_KEY') continue;
    if (!password && /(KEY|PASS|PASSWORD|SECRET|TOKEN)/.test(upper)) {
      password = val;
    } else if (!username && /(USER|USERNAME|LOGIN)/.test(upper)) {
      username = val;
    } else if (!senderEmail && /(SENDER|FROM|EMAIL|MAIL)/.test(upper)) {
      senderEmail = val;
    } else if (!host && /HOST|SERVER|RELAY/.test(upper)) {
      host = val;
    } else if (!port && /PORT/.test(upper)) {
      port = val;
    }
  }

  const brevoApiKey = process.env.BREVO_API_KEY?.trim();
  const needsBrevoLookup = !username || !senderEmail;
  if (brevoApiKey && needsBrevoLookup) {
    try {
      const accountRes = await fetch('https://api.brevo.com/v3/account', {
        headers: { 'api-key': brevoApiKey, Accept: 'application/json' },
      });
      if (accountRes.ok) {
        neonEmailSyncStatus.brevoApiAuthorized = true;
        const account = (await accountRes.json()) as {
          email?: string;
          relay?: {
            enabled?: boolean;
            data?: { userName?: string; relay?: string; port?: number };
          };
        };
        if (!username && account.relay?.data?.userName) {
          username = account.relay.data.userName.trim();
        }
        if (!host && account.relay?.data?.relay) {
          host = account.relay.data.relay.trim();
        }
        if (!port && account.relay?.data?.port) {
          port = String(account.relay.data.port);
        }
        if (!senderEmail && account.email) {
          senderEmail = account.email.trim();
        }
      } else {
        neonEmailSyncStatus.brevoApiAuthorized = false;
      }
    } catch {
      neonEmailSyncStatus.brevoApiAuthorized = false;
    }

    if (!senderEmail) {
      try {
        const sendersRes = await fetch('https://api.brevo.com/v3/senders', {
          headers: { 'api-key': brevoApiKey, Accept: 'application/json' },
        });
        if (sendersRes.ok) {
          const sendersPayload = (await sendersRes.json()) as {
            senders?: Array<{ email?: string; name?: string; active?: boolean }>;
          };
          const activeSender =
            sendersPayload.senders?.find((s) => s.active && s.email) ||
            sendersPayload.senders?.find((s) => s.email);
          if (activeSender?.email) {
            senderEmail = activeSender.email.trim();
            if (activeSender.name?.trim()) {
              senderName = activeSender.name.trim();
            }
          }
        }
      } catch {
        // Fallback to account email or env value
      }
    }
  } else if (!needsBrevoLookup) {
    neonEmailSyncStatus.brevoApiAuthorized = null;
  }

  return {
    host: host || 'smtp-relay.brevo.com',
    port: port || '587',
    username,
    password,
    senderEmail: senderEmail || getAdminEmail(),
    senderName,
  };
}

async function syncEmailProviderInBackground() {
  if (!process.env.NEON_API_KEY) return;
  const previousCfg = await getEmailProviderConfig();
  const cfg = await resolveBrevoConfigFromEnvAndApi();
  if (!cfg.username || !cfg.password) {
    return;
  }

  const args = [
    'neon-auth',
    'config',
    'email-provider',
    'update',
    '--type',
    'standard',
    '--host',
    cfg.host,
    '--port',
    String(cfg.port),
    '--username',
    cfg.username,
    '--password',
    cfg.password,
    '--sender-name',
    cfg.senderName,
  ];
  if (cfg.senderEmail) {
    args.push('--sender-email', cfg.senderEmail);
  }

  execFile(
    resolveNeonBin(),
    args,
    { cwd: ROOT_DIR, env: process.env, timeout: 25000 },
    async (err, _stdout, stderr) => {
      if (!err) {
        console.log('Synced Brevo SMTP email provider to Neon Auth.');
        neonEmailSyncStatus.synced = true;
        neonEmailSyncStatus.lastError = null;
        await getEmailProviderConfig();

        if (
          previousCfg?.type !== 'standard' ||
          previousCfg?.sender_email !== cfg.senderEmail
        ) {
          execFile(
            resolveNeonBin(),
            [
              'neon-auth',
              'config',
              'email-provider',
              'test',
              '--recipient-email',
              getAdminEmail(),
            ],
            { cwd: ROOT_DIR, env: process.env, timeout: 25000 },
            (testErr, _testOut, testStderr) => {
              if (!testErr) {
                neonEmailSyncStatus.testResult = 'sent';
                console.log('Verified Brevo custom SMTP test delivery via Neon Auth.');
              } else {
                neonEmailSyncStatus.testResult = (
                  testStderr || testErr.message || 'test failed'
                ).slice(0, 200);
              }
            }
          );
        }
      } else {
        const safeMsg = (stderr || err.message || '').slice(0, 200);
        neonEmailSyncStatus.lastError = safeMsg;
        console.error('Unable to sync Brevo email provider:', safeMsg);
        await getEmailProviderConfig();
      }
    }
  );
}

function ensureLocalDidKeyEncryptionSecret() {
  if (process.env.DID_KEY_ENCRYPTION_SECRET) {
    return;
  }
  const secret = crypto.randomBytes(32).toString('hex');
  process.env.DID_KEY_ENCRYPTION_SECRET = secret;
  try {
    const existing = fs.existsSync(envLocalPath)
      ? fs.readFileSync(envLocalPath, 'utf-8')
      : '';
    if (!/^DID_KEY_ENCRYPTION_SECRET=/m.test(existing)) {
      const separator = existing.length > 0 && !existing.endsWith('\n') ? '\n' : '';
      fs.appendFileSync(
        envLocalPath,
        `${separator}DID_KEY_ENCRYPTION_SECRET=${secret}\n`,
        'utf-8'
      );
    }
  } catch (err) {
    console.warn('Unable to persist DID_KEY_ENCRYPTION_SECRET to .env.local:', err);
  }
}

async function startServer() {
  bootstrapNeonDeployIfNeeded();
  ensureLocalDidKeyEncryptionSecret();

  const app = express();
  app.set('trust proxy', 1);
  app.use(express.json());

  if (isPostgresConfigured()) {
    try {
      await ensureSchema();
    } catch (err) {
      console.error('Database schema initialization warning:', err);
    }
  }

  app.get('/api/health', async (_req, res) => {
    let dbConnected = false;
    if (isPostgresConfigured()) {
      try {
        await getPool().query('SELECT 1');
        dbConnected = true;
      } catch {
        dbConnected = false;
      }
    }

    res.setHeader('Cache-Control', 'no-store');
    res.json({
      ok: true,
      hasNeonApiKey: Boolean(process.env.NEON_API_KEY),
      hasGoogleOAuthSecrets: Boolean(
        process.env.GOOGLE_OAUTH_CLIENT_ID && process.env.GOOGLE_OAUTH_CLIENT_SECRET
      ),
      neonDeployStatus,
      neonOAuthSyncStatus,
      neonEmailSyncStatus,
      services: {
        postgres: dbConnected,
        auth: isAuthConfigured(),
        storage: isStorageConfigured(),
      },
    });
  });

  app.get('/api/auth-config', (_req, res) => {
    const authUrl =
      process.env.NEON_AUTH_BASE_URL || process.env.VITE_NEON_AUTH_URL || null;
    res.setHeader('Cache-Control', 'no-store');
    res.json({
      authConfigured: isAuthConfigured() && Boolean(authUrl),
      authUrl,
      adminEmail: getAdminEmail(),
    });
  });

  app.get('/api/me', requireAuthenticatedUser, (req: AuthenticatedRequest, res) => {
    res.setHeader('Cache-Control', 'no-store');
    res.json({
      authenticated: true,
      user: req.authUser,
    });
  });

  app.get(
    '/api/docs',
    requireAuthenticatedUser,
    requireDocsAccess,
    (_req, res) => {
      try {
        const tree = getDocsTree();
        res.setHeader('Cache-Control', 'no-store');
        res.json({ tree });
      } catch (err) {
        console.error('Failed to list documentation files:', err);
        res.status(500).json({ error: 'Unable to read documentation directory.' });
      }
    }
  );

  app.get(
    '/api/doc',
    requireAuthenticatedUser,
    requireDocsAccess,
    (req, res) => {
      const rawPath = typeof req.query.path === 'string' ? req.query.path : '';
      const absPath = resolveSafeDocPath(rawPath);

      if (!absPath) {
        res.status(404).json({ error: 'Document not found.' });
        return;
      }

      try {
        const stat = fs.statSync(absPath);
        const content = fs.readFileSync(absPath, 'utf-8');
        const fileName = path.basename(absPath);
        const { title, headings, wordCount } = extractTitleAndHeadings(content, fileName);
        const readingMinutes = Math.max(1, Math.ceil(wordCount / 210));
        const relPath = path.relative(ROOT_DIR, absPath).replace(/\\/g, '/');

        res.setHeader('Cache-Control', 'no-store');
        res.json({
          path: relPath,
          name: fileName,
          title,
          content,
          updatedAt: stat.mtime.toISOString(),
          wordCount,
          readingMinutes,
          headings,
        });
      } catch (err) {
        console.error('Failed to read document:', err);
        res.status(500).json({ error: 'Unable to read requested document.' });
      }
    }
  );

  app.get(
    '/api/admin/allowlist',
    requireAuthenticatedUser,
    requireAdminAccess,
    async (_req, res) => {
      try {
        const entries = await listAllowlist();
        res.json({ entries });
      } catch (err) {
        console.error('Failed to load allowlist:', err);
        res.status(500).json({ error: 'Failed to load access allowlist.' });
      }
    }
  );

  app.post(
    '/api/admin/allowlist',
    requireAuthenticatedUser,
    requireAdminAccess,
    async (req: AuthenticatedRequest, res) => {
      const email = typeof req.body?.email === 'string' ? req.body.email.trim() : '';
      const enabled = typeof req.body?.enabled === 'boolean' ? req.body.enabled : true;
      const note = typeof req.body?.note === 'string' ? req.body.note : '';

      if (!email || !email.includes('@')) {
        res.status(400).json({ error: 'Valid email address is required.' });
        return;
      }

      try {
        const entry = await upsertAllowlistEntry({
          email,
          enabled,
          note,
          addedBy: req.authUser?.email || getAdminEmail(),
        });
        res.json({ entry });
      } catch (err) {
        console.error('Failed to update allowlist:', err);
        res.status(500).json({ error: 'Failed to save access entry.' });
      }
    }
  );

  app.delete(
    '/api/admin/allowlist',
    requireAuthenticatedUser,
    requireAdminAccess,
    async (req, res) => {
      const email = typeof req.query.email === 'string' ? req.query.email.trim() : '';
      if (!email) {
        res.status(400).json({ error: 'Email is required.' });
        return;
      }

      try {
        const removed = await deleteAllowlistEntry(email);
        res.json({ removed });
      } catch (err) {
        res.status(400).json({
          error: err instanceof Error ? err.message : 'Unable to remove entry.',
        });
      }
    }
  );

  app.get(
    '/api/admin/identities',
    requireAuthenticatedUser,
    requireAdminAccess,
    async (_req, res) => {
      try {
        const identities = await listIdentities();
        res.json({ identities });
      } catch (err) {
        console.error('Failed to list identities:', err);
        res.status(500).json({ error: 'Failed to load user identities.' });
      }
    }
  );

  app.get(
    '/api/storage/snapshots',
    requireAuthenticatedUser,
    requireAdminAccess,
    async (_req, res) => {
      if (!isStorageConfigured()) {
        res.status(503).json({ error: 'Neon Object Storage is not configured.' });
        return;
      }
      try {
        const rows = await listSnapshots();
        const snapshots = await Promise.all(
          rows.map(async (row) => ({
            ...row,
            downloadUrl: await getSnapshotPresignedUrl(row.objectKey),
          }))
        );
        res.json({ snapshots });
      } catch (err) {
        console.error('Failed to list snapshots:', err);
        res.status(500).json({ error: 'Unable to list storage snapshots.' });
      }
    }
  );

  app.post(
    '/api/storage/snapshot',
    requireAuthenticatedUser,
    requireAdminAccess,
    async (req: AuthenticatedRequest, res) => {
      if (!isStorageConfigured()) {
        res.status(503).json({ error: 'Neon Object Storage is not configured.' });
        return;
      }
      const docPath = typeof req.body?.docPath === 'string' ? req.body.docPath : '';
      const absPath = resolveSafeDocPath(docPath);
      if (!absPath) {
        res.status(404).json({ error: 'Document not found.' });
        return;
      }

      try {
        const content = fs.readFileSync(absPath, 'utf-8');
        const uploaded = await uploadDocSnapshot({ docPath, content });
        await recordSnapshot({
          docPath,
          objectKey: uploaded.objectKey,
          sizeBytes: uploaded.sizeBytes,
          uploadedByDid: req.authUser?.didPeer || req.authUser?.authUserId || 'admin',
        });
        const downloadUrl = await getSnapshotPresignedUrl(uploaded.objectKey);
        res.json({
          objectKey: uploaded.objectKey,
          sizeBytes: uploaded.sizeBytes,
          downloadUrl,
        });
      } catch (err) {
        console.error('Failed to archive snapshot:', err);
        res.status(500).json({ error: 'Unable to upload snapshot to Neon Object Storage.' });
      }
    }
  );

  if (process.env.NODE_ENV === 'production') {
    const distDir = path.join(ROOT_DIR, 'dist');
    app.use(express.static(distDir));
    app.get('*', (_req, res) => {
      res.sendFile(path.join(distDir, 'index.html'));
    });
  } else {
    const vite = await createViteServer({
      server: { middlewareMode: true },
      appType: 'spa',
    });
    app.use(vite.middlewares);
  }

  app.listen(PORT, '0.0.0.0', () => {
    console.log(`ExoNovel documentation server listening on http://0.0.0.0:${PORT}`);
    syncTrustedOriginsInBackground();
    syncOAuthProvidersInBackground();
    syncEmailProviderInBackground();
  });
}

startServer();
