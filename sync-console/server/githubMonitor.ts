import type { Request, Response } from 'express';
import {
  getGitHubToken,
  GITHUB_BASE_BRANCH,
  GITHUB_OWNER,
  GITHUB_REPO,
} from './githubSync.ts';

const GITHUB_API_BASE = 'https://api.github.com';
const POLL_INTERVAL_MS = 12_000;
const HEARTBEAT_INTERVAL_MS = 25_000;

interface WatchedEndpoint {
  key: string;
  url: string;
  etag: string | null;
  fingerprint: string | null;
}

const watchedEndpoints: WatchedEndpoint[] = [
  {
    key: 'branch',
    url: `${GITHUB_API_BASE}/repos/${GITHUB_OWNER}/${GITHUB_REPO}/branches/${GITHUB_BASE_BRANCH}`,
    etag: null,
    fingerprint: null,
  },
  {
    key: 'pulls',
    url: `${GITHUB_API_BASE}/repos/${GITHUB_OWNER}/${GITHUB_REPO}/pulls?state=open&per_page=20`,
    etag: null,
    fingerprint: null,
  },
  {
    key: 'checks',
    url: `${GITHUB_API_BASE}/repos/${GITHUB_OWNER}/${GITHUB_REPO}/commits/${GITHUB_BASE_BRANCH}/check-runs?per_page=20`,
    etag: null,
    fingerprint: null,
  },
];

const subscribers = new Set<Response>();
let pollTimer: NodeJS.Timeout | null = null;
let heartbeatTimer: NodeJS.Timeout | null = null;
let pollInFlight = false;

function broadcastEvent(eventName: string, payload: Record<string, unknown>): void {
  const data = `event: ${eventName}\ndata: ${JSON.stringify(payload)}\n\n`;
  for (const res of subscribers) {
    try {
      res.write(data);
    } catch {
      subscribers.delete(res);
    }
  }
}

function broadcastPing(): void {
  for (const res of subscribers) {
    try {
      res.write(': keepalive\n\n');
    } catch {
      subscribers.delete(res);
    }
  }
}

async function checkEndpoint(ep: WatchedEndpoint, token: string): Promise<boolean> {
  const headers: Record<string, string> = {
    Authorization: `Bearer ${token}`,
    Accept: 'application/vnd.github+json',
    'X-GitHub-Api-Version': '2022-11-28',
    'User-Agent': 'ExoNovel-Workspace-Sync',
  };
  if (ep.etag) {
    headers['If-None-Match'] = ep.etag;
  }

  const res = await fetch(ep.url, {
    headers,
    signal: AbortSignal.timeout(8000),
  });

  if (res.status === 304) {
    return false;
  }
  if (!res.ok) {
    return false;
  }

  const nextEtag = res.headers.get('etag');
  const bodyText = await res.text().catch(() => '');
  const prevEtag = ep.etag;
  const prevFingerprint = ep.fingerprint;

  let nextFingerprint = bodyText.slice(0, 200);
  try {
    const parsed = JSON.parse(bodyText) as Record<string, unknown>;
    if (ep.key === 'branch' && parsed.commit && typeof parsed.commit === 'object') {
      nextFingerprint = String((parsed.commit as { sha?: unknown }).sha || '');
    } else if (ep.key === 'pulls' && Array.isArray(parsed)) {
      nextFingerprint = parsed
        .map((p: { number?: number; head?: { sha?: string } }) =>
          `${p.number ?? ''}:${p.head?.sha ?? ''}`
        )
        .join('|');
    } else if (ep.key === 'checks' && Array.isArray(parsed.check_runs)) {
      nextFingerprint = (
        parsed.check_runs as Array<{
          id?: number;
          status?: string;
          conclusion?: string | null;
        }>
      )
        .map((c) => `${c.id ?? ''}:${c.status ?? ''}:${c.conclusion ?? ''}`)
        .join('|');
    }
  } catch {
    // Keep body prefix fallback
  }

  ep.etag = nextEtag;
  ep.fingerprint = nextFingerprint;

  if (prevEtag === null && prevFingerprint === null) {
    return false;
  }
  return prevFingerprint !== nextFingerprint || prevEtag !== nextEtag;
}

async function pollGitHubChanges(): Promise<void> {
  if (subscribers.size === 0 || pollInFlight) return;
  const token = getGitHubToken();
  if (!token) return;

  pollInFlight = true;
  try {
    const changedKeys: string[] = [];
    for (const ep of watchedEndpoints) {
      try {
        const changed = await checkEndpoint(ep, token);
        if (changed) changedKeys.push(ep.key);
      } catch {
        // Skip transient network error and keep polling
      }
    }

    if (changedKeys.length > 0) {
      broadcastEvent('update', {
        source: 'github',
        changed: changedKeys,
        at: new Date().toISOString(),
      });
    }
  } finally {
    pollInFlight = false;
  }
}

function ensureMonitorLoops(): void {
  if (!pollTimer) {
    pollTimer = setInterval(() => {
      void pollGitHubChanges();
    }, POLL_INTERVAL_MS);
    void pollGitHubChanges();
  }
  if (!heartbeatTimer) {
    heartbeatTimer = setInterval(() => {
      broadcastPing();
    }, HEARTBEAT_INTERVAL_MS);
  }
}

function stopMonitorLoopsIfIdle(): void {
  if (subscribers.size > 0) return;
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
  if (heartbeatTimer) {
    clearInterval(heartbeatTimer);
    heartbeatTimer = null;
  }
}

export function subscribeGitHubEvents(req: Request, res: Response): void {
  res.setHeader('Content-Type', 'text/event-stream; charset=utf-8');
  res.setHeader('Cache-Control', 'no-cache, no-transform');
  res.setHeader('Connection', 'keep-alive');
  res.setHeader('X-Accel-Buffering', 'no');
  res.flushHeaders?.();

  subscribers.add(res);
  res.write(
    `event: connected\ndata: ${JSON.stringify({ at: new Date().toISOString() })}\n\n`
  );
  ensureMonitorLoops();

  req.on('close', () => {
    subscribers.delete(res);
    stopMonitorLoopsIfIdle();
  });
}

export function notifyWorkspaceChange(reason: string): void {
  for (const ep of watchedEndpoints) {
    ep.etag = null;
  }
  if (subscribers.size > 0) {
    broadcastEvent('update', {
      source: 'workspace',
      reason,
      at: new Date().toISOString(),
    });
  }
}
