import dotenv from 'dotenv';
import express from 'express';
import fs from 'node:fs';
import path from 'node:path';
import { createServer as createViteServer } from 'vite';
import { notifyWorkspaceChange, subscribeGitHubEvents } from './githubMonitor.ts';
import {
  addGitignorePattern,
  computeWorkspaceDiff,
  getGitHubSyncStatus,
  getWorkspaceFileDiff,
  GitHubSyncConflictError,
  mergeGitHubPullRequest,
  probeProductionSite,
  pullFilesFromGitHub,
  removeGitignorePattern,
  resolveWorkspaceMerge,
  syncWorkspaceToGitHub,
  unignoreFileInGitignore,
} from './githubSync.ts';
import {
  claimWorkspacePasscode,
  clearWorkspaceSessionCookie,
  getWorkspaceAuthStatus,
  hasValidCsrfGuard,
  requireAiStudioEnvironment,
  requireWorkspaceSession,
  setWorkspaceSessionCookie,
  unlockWithWorkspacePasscode,
} from './workspaceAuth.ts';

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

async function startServer() {
  const app = express();
  app.set('trust proxy', 1);
  app.use(express.json({ limit: '2mb' }));

  app.get('/api/health', (_req, res) => {
    res.setHeader('Cache-Control', 'no-store');
    res.json({
      ok: true,
      mode: 'sync-console',
      githubConfigured: Boolean(process.env.GITHUB_PAT?.trim()),
      passcodeConfigured: Boolean(process.env.ADMIN_PASSCODE?.trim()),
    });
  });

  app.use(requireAiStudioEnvironment);

  app.get('/api/workspace-auth/status', async (req, res) => {
    try {
      const status = await getWorkspaceAuthStatus(req);
      res.setHeader('Cache-Control', 'no-store');
      res.json(status);
    } catch (err) {
      console.error('Failed to read workspace auth status:', err);
      res.status(500).json({ error: 'Unable to check workspace passcode status.' });
    }
  });

  app.post('/api/workspace-auth/claim', async (req, res) => {
    if (!hasValidCsrfGuard(req)) {
      res.status(403).json({ error: 'Invalid request origin or missing sync header.' });
      return;
    }
    const passcode = typeof req.body?.passcode === 'string' ? req.body.passcode : '';
    try {
      const token = await claimWorkspacePasscode(passcode);
      setWorkspaceSessionCookie(req, res, token);
      res.setHeader('Cache-Control', 'no-store');
      res.json({ token });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Unable to set workspace passcode.';
      const status = message.includes('already') ? 409 : 400;
      res.status(status).json({ error: message });
    }
  });

  app.post('/api/workspace-auth/unlock', async (req, res) => {
    if (!hasValidCsrfGuard(req)) {
      res.status(403).json({ error: 'Invalid request origin or missing sync header.' });
      return;
    }
    const passcode = typeof req.body?.passcode === 'string' ? req.body.passcode : '';
    try {
      const token = await unlockWithWorkspacePasscode(passcode, req);
      setWorkspaceSessionCookie(req, res, token);
      res.setHeader('Cache-Control', 'no-store');
      res.json({ token });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Invalid workspace passcode.';
      const status = message.includes('Too many') ? 429 : 401;
      res.status(status).json({ error: message });
    }
  });

  app.post('/api/workspace-auth/lock', (req, res) => {
    if (!hasValidCsrfGuard(req)) {
      res.status(403).json({ error: 'Invalid request origin or missing sync header.' });
      return;
    }
    clearWorkspaceSessionCookie(req, res);
    res.setHeader('Cache-Control', 'no-store');
    res.json({ locked: true });
  });

  app.get('/api/admin/github/events', requireWorkspaceSession, (req, res) => {
    subscribeGitHubEvents(req, res);
  });

  app.get('/api/admin/github/status', requireWorkspaceSession, async (_req, res) => {
    try {
      const status = await getGitHubSyncStatus();
      res.setHeader('Cache-Control', 'no-store');
      res.json(status);
    } catch (err) {
      console.error('Failed to check GitHub status:', err);
      res.status(500).json({ error: 'Unable to check GitHub repository status.' });
    }
  });

  app.get(
    '/api/admin/github/prod-probe',
    requireWorkspaceSession,
    async (_req, res) => {
      try {
        const probe = await probeProductionSite();
        res.setHeader('Cache-Control', 'no-store');
        res.json(probe);
      } catch (err) {
        res.status(502).json({
          error:
            err instanceof Error ? err.message : 'Unable to probe production website.',
        });
      }
    }
  );

  app.get('/api/admin/github/diff', requireWorkspaceSession, async (_req, res) => {
    try {
      const diff = await computeWorkspaceDiff();
      res.setHeader('Cache-Control', 'no-store');
      res.json(diff);
    } catch (err) {
      res.status(502).json({
        error:
          err instanceof Error
            ? err.message
            : 'Unable to compare workspace against GitHub repository.',
      });
    }
  });

  app.get(
    '/api/admin/github/file-diff',
    requireWorkspaceSession,
    async (req, res) => {
      const rawPath = typeof req.query?.path === 'string' ? req.query.path : '';
      if (!rawPath) {
        res.status(400).json({ error: 'Query parameter "path" is required.' });
        return;
      }
      try {
        const detail = await getWorkspaceFileDiff(rawPath);
        res.setHeader('Cache-Control', 'no-store');
        res.json(detail);
      } catch (err) {
        res.status(400).json({
          error: err instanceof Error ? err.message : 'Unable to load file diff.',
        });
      }
    }
  );

  app.post('/api/admin/github/gitignore', requireWorkspaceSession, (req, res) => {
    const action = req.body?.action;
    const value = typeof req.body?.value === 'string' ? req.body.value : '';

    try {
      res.setHeader('Cache-Control', 'no-store');
      if (action === 'add') {
        const result = addGitignorePattern(value);
        notifyWorkspaceChange('gitignore');
        res.json({ ok: true, ...result });
        return;
      }
      if (action === 'remove') {
        const result = removeGitignorePattern(value);
        notifyWorkspaceChange('gitignore');
        res.json({ ok: true, ...result });
        return;
      }
      if (action === 'unignore') {
        const result = unignoreFileInGitignore(value);
        notifyWorkspaceChange('gitignore');
        res.json({ ok: true, ...result });
        return;
      }
      res.status(400).json({ error: 'Invalid gitignore action.' });
    } catch (err) {
      res.status(400).json({
        error: err instanceof Error ? err.message : 'Unable to update .gitignore.',
      });
    }
  });

  app.post('/api/admin/github/pull', requireWorkspaceSession, async (req, res) => {
    const paths = Array.isArray(req.body?.paths)
      ? req.body.paths.filter((p: unknown): p is string => typeof p === 'string')
      : [];
    const overwritePaths = Array.isArray(req.body?.overwritePaths)
      ? req.body.overwritePaths.filter(
          (p: unknown): p is string => typeof p === 'string'
        )
      : undefined;
    const expectedHeadSha =
      typeof req.body?.expectedHeadSha === 'string'
        ? req.body.expectedHeadSha
        : undefined;

    try {
      const result = await pullFilesFromGitHub({
        paths,
        expectedHeadSha,
        overwritePaths,
      });
      notifyWorkspaceChange('pull');
      res.setHeader('Cache-Control', 'no-store');
      res.json(result);
    } catch (err) {
      if (err instanceof GitHubSyncConflictError) {
        res.status(409).json({
          error: err.message,
          conflicts: err.conflicts,
          headSha: err.headSha,
        });
        return;
      }
      res.status(400).json({
        error:
          err instanceof Error ? err.message : 'Unable to pull files from GitHub.',
      });
    }
  });

  app.post(
    '/api/admin/github/resolve-merge',
    requireWorkspaceSession,
    async (req, res) => {
      const filePath = typeof req.body?.path === 'string' ? req.body.path : '';
      const resolvedContent =
        typeof req.body?.resolvedContent === 'string'
          ? req.body.resolvedContent
          : undefined;
      const remoteSha =
        typeof req.body?.remoteSha === 'string' ? req.body.remoteSha : '';
      const rawAction = req.body?.action;
      const action =
        rawAction === 'delete_local' || rawAction === 'keep_local'
          ? rawAction
          : 'save';

      if (
        !filePath ||
        (action === 'save' && (resolvedContent === undefined || !remoteSha))
      ) {
        res.status(400).json({
          error:
            action === 'save'
              ? 'Fields "path", "resolvedContent", and "remoteSha" are required.'
              : 'Field "path" is required.',
        });
        return;
      }

      try {
        const result = await resolveWorkspaceMerge({
          path: filePath,
          resolvedContent,
          remoteSha,
          action,
        });
        notifyWorkspaceChange('resolve-merge');
        res.setHeader('Cache-Control', 'no-store');
        res.json({ ok: true, ...result });
      } catch (err) {
        res.status(400).json({
          error: err instanceof Error ? err.message : 'Unable to resolve merge.',
        });
      }
    }
  );

  app.post('/api/admin/github/sync', requireWorkspaceSession, async (req, res) => {
    const mode = req.body?.mode === 'direct' ? 'direct' : 'pr';
    const paths = Array.isArray(req.body?.paths)
      ? req.body.paths.filter((p: unknown): p is string => typeof p === 'string')
      : [];
    const deletePaths = Array.isArray(req.body?.deletePaths)
      ? req.body.deletePaths.filter((p: unknown): p is string => typeof p === 'string')
      : [];
    const commitMessage =
      typeof req.body?.commitMessage === 'string' ? req.body.commitMessage : '';
    const branchName =
      typeof req.body?.branchName === 'string' ? req.body.branchName : undefined;
    const prTitle = typeof req.body?.prTitle === 'string' ? req.body.prTitle : undefined;
    const prBody = typeof req.body?.prBody === 'string' ? req.body.prBody : undefined;
    const expectedHeadSha =
      typeof req.body?.expectedHeadSha === 'string'
        ? req.body.expectedHeadSha
        : undefined;
    const rawKnownShas = req.body?.knownRemoteShas;
    const knownRemoteShas: Record<string, string> = {};
    if (rawKnownShas && typeof rawKnownShas === 'object' && !Array.isArray(rawKnownShas)) {
      for (const [k, v] of Object.entries(rawKnownShas as Record<string, unknown>)) {
        if (typeof v === 'string') {
          knownRemoteShas[k] = v;
        }
      }
    }

    try {
      const result = await syncWorkspaceToGitHub({
        mode,
        paths,
        deletePaths,
        commitMessage,
        branchName,
        prTitle,
        prBody,
        expectedHeadSha,
        knownRemoteShas,
      });
      notifyWorkspaceChange('sync');
      res.json(result);
    } catch (err) {
      if (err instanceof GitHubSyncConflictError) {
        res.status(409).json({
          error: err.message,
          conflicts: err.conflicts,
          headSha: err.headSha,
        });
        return;
      }
      res.status(400).json({
        error: err instanceof Error ? err.message : 'GitHub repository sync failed.',
      });
    }
  });

  app.post('/api/admin/github/merge', requireWorkspaceSession, async (req, res) => {
    const pullNumber = Number(req.body?.pullNumber);

    try {
      const result = await mergeGitHubPullRequest({
        pullNumber,
        mergeMethod: 'merge',
      });
      notifyWorkspaceChange('merge');
      res.json(result);
    } catch (err) {
      res.status(400).json({
        error: err instanceof Error ? err.message : 'Unable to merge pull request.',
      });
    }
  });

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
    console.log(`ExoNovel Sync Console listening on http://0.0.0.0:${PORT}`);
  });
}

startServer();
