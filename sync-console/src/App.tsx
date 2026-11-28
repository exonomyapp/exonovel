/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import { useCallback, useEffect, useRef, useState, type FormEvent } from 'react';
import { Lock, Moon, RefreshCw, ShieldAlert, Sun } from 'lucide-react';
import { GitHubSyncPanel } from './components/GitHubSyncPanel';

interface WorkspaceAuthStatus {
  claimed: boolean;
  source: 'env' | 'claimed' | 'unclaimed';
  unlocked: boolean;
  aiStudioRestricted?: boolean;
}

export default function App() {
  const [authStatus, setAuthStatus] = useState<WorkspaceAuthStatus | null>(null);
  const [passcode, setPasscode] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  const [diffRefreshing, setDiffRefreshing] = useState(false);
  const memoryTokenRef = useRef<string | null>(null);
  const [darkMode, setDarkMode] = useState<boolean>(() => {
    try {
      return window.localStorage.getItem('exonovel-sync-theme') === 'dark';
    } catch {
      return false;
    }
  });

  useEffect(() => {
    const root = document.documentElement;
    if (darkMode) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
    try {
      window.localStorage.setItem('exonovel-sync-theme', darkMode ? 'dark' : 'light');
    } catch {
      // Ignore storage failures
    }
  }, [darkMode]);

  const loadAuthStatus = useCallback(async () => {
    try {
      const res = await fetch('/api/workspace-auth/status', {
        credentials: 'include',
        cache: 'no-store',
        headers: memoryTokenRef.current
          ? { Authorization: `Bearer ${memoryTokenRef.current}` }
          : undefined,
      });
      if (res.status === 403) {
        const body = await res.json().catch(() => ({}));
        setAuthStatus({
          claimed: true,
          source: 'env',
          unlocked: false,
          aiStudioRestricted: true,
        });
        setError(
          body.error || 'Sync Console is restricted to AI Studio preview workspaces.'
        );
        return;
      }
      if (!res.ok) {
        throw new Error('Unable to verify workspace session.');
      }
      const data = (await res.json()) as WorkspaceAuthStatus;
      setAuthStatus(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to reach workspace server.');
      setAuthStatus({ claimed: true, source: 'env', unlocked: false });
    }
  }, []);

  useEffect(() => {
    loadAuthStatus();
  }, [loadAuthStatus]);

  const getToken = useCallback(async () => memoryTokenRef.current, []);

  const handleUnauthorized = useCallback(() => {
    memoryTokenRef.current = null;
    setAuthStatus((prev) => (prev ? { ...prev, unlocked: false } : null));
  }, []);

  const handlePasscodeSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!passcode.trim() || !authStatus) return;

    setSubmitting(true);
    setError(null);
    const endpoint = authStatus.claimed
      ? '/api/workspace-auth/unlock'
      : '/api/workspace-auth/claim';

    try {
      const res = await fetch(endpoint, {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
          'X-Requested-With': 'exonovel-sync',
        },
        body: JSON.stringify({ passcode }),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        throw new Error(data.error || 'Unable to unlock workspace.');
      }
      if (typeof data.token === 'string' && data.token) {
        memoryTokenRef.current = data.token;
      }
      setPasscode('');
      await loadAuthStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unlock failed.');
    } finally {
      setSubmitting(false);
    }
  };

  const handleLock = async () => {
    try {
      await fetch('/api/workspace-auth/lock', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
          'X-Requested-With': 'exonovel-sync',
          ...(memoryTokenRef.current
            ? { Authorization: `Bearer ${memoryTokenRef.current}` }
            : {}),
        },
      });
    } finally {
      memoryTokenRef.current = null;
      await loadAuthStatus();
    }
  };

  return (
    <div className="min-h-screen flex flex-col bg-[#FAF8F4] dark:bg-[#111317] text-[#1C1917] dark:text-[#ECEEF2]">
      <header className="sticky top-0 z-30 h-14 px-4 sm:px-8 border-b border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4]/95 dark:bg-[#111317]/95 backdrop-blur-sm flex items-center justify-between gap-4 sm:gap-8">
        <span className="font-display text-base sm:text-lg font-semibold tracking-tight text-[#1C1917] dark:text-[#ECEEF2] whitespace-nowrap truncate">
          Update{' '}
          <span className="font-mono text-xs sm:text-sm font-medium">
            exonomyapp/exonovel
          </span>
        </span>

        <div className="flex items-center gap-2 shrink-0">
          {authStatus?.unlocked && (
            <button
              type="button"
              onClick={() => setRefreshTrigger((prev) => prev + 1)}
              disabled={diffRefreshing}
              className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] text-[#1C1917] dark:text-[#ECEEF2] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors whitespace-nowrap shrink-0 min-h-[40px] disabled:opacity-50"
              aria-label="Refresh GitHub diff"
            >
              <RefreshCw
                className={`w-3.5 h-3.5 ${diffRefreshing ? 'animate-spin' : ''}`}
              />
              <span>
                {diffRefreshing ? (
                  'Checking...'
                ) : (
                  <>
                    Refresh<span className="hidden sm:inline"> Diff</span>
                  </>
                )}
              </span>
            </button>
          )}

          <button
            type="button"
            onClick={() => setDarkMode((prev) => !prev)}
            className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] text-[#1C1917] dark:text-[#ECEEF2] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors whitespace-nowrap shrink-0 min-h-[40px]"
            aria-label={darkMode ? 'Switch to light theme' : 'Switch to dark theme'}
          >
            {darkMode ? (
              <Sun className="w-3.5 h-3.5" />
            ) : (
              <Moon className="w-3.5 h-3.5" />
            )}
            <span>{darkMode ? 'Light' : 'Dark'}</span>
          </button>

          {authStatus?.unlocked && (
            <button
              type="button"
              onClick={handleLock}
              className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] text-[#1C1917] dark:text-[#ECEEF2] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors whitespace-nowrap shrink-0 min-h-[40px]"
              aria-label="Lock workspace"
            >
              <Lock className="w-3.5 h-3.5" />
              <span className="hidden sm:inline">Lock</span>
            </button>
          )}
        </div>
      </header>

      <main className="flex-1 w-full max-w-5xl mx-auto px-5 sm:px-8 py-8 sm:py-10">
        {!authStatus ? (
          <div className="py-16 text-sm text-[#57534E] dark:text-[#9CA3AF]">
            Loading workspace session...
          </div>
        ) : authStatus.aiStudioRestricted ? (
          <div className="max-w-md mx-auto mt-10 rounded-lg border border-red-300 dark:border-red-900/60 bg-[#FFFFFF] dark:bg-[#171A21] p-6 sm:p-8">
            <div className="flex items-center gap-2 text-red-700 dark:text-red-400">
              <ShieldAlert className="w-5 h-5 shrink-0" />
              <h1 className="font-display text-xl font-semibold tracking-tight">
                AI Studio Access Only
              </h1>
            </div>
            <p className="mt-2 text-sm text-[#57534E] dark:text-[#9CA3AF]">
              {error ||
                'The ExoNovel Sync Console is locked to AI Studio preview workspaces and cannot be accessed from external hosts.'}
            </p>
          </div>
        ) : authStatus.unlocked ? (
          <GitHubSyncPanel
            getToken={getToken}
            refreshTrigger={refreshTrigger}
            onLoadingChange={setDiffRefreshing}
            onUnauthorized={handleUnauthorized}
          />
        ) : (
          <div className="max-w-md mx-auto mt-10 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] p-6 sm:p-8">
            <h1 className="font-display text-2xl font-semibold tracking-tight">
              {authStatus.claimed ? 'Unlock Sync Console' : 'Set Sync Passcode'}
            </h1>
            <p className="mt-2 text-sm text-[#57534E] dark:text-[#9CA3AF]">
              {authStatus.claimed
                ? 'Enter your workspace passcode to review diffs, pull main updates, and push changes to exonomyapp/exonovel.'
                : 'Choose a workspace passcode to protect GitHub sync actions. Your browser stays unlocked for 7 days.'}
            </p>

            {error && (
              <div className="mt-4 p-3 rounded-md border border-red-300 dark:border-red-900/60 bg-red-50 dark:bg-red-950/40 text-xs text-red-700 dark:text-red-300">
                {error}
              </div>
            )}

            <form onSubmit={handlePasscodeSubmit} className="mt-5 space-y-4">
              <div>
                <label
                  htmlFor="workspace-passcode"
                  className="block text-xs font-medium text-[#57534E] dark:text-[#9CA3AF] mb-1.5"
                >
                  Workspace Passcode
                </label>
                <input
                  id="workspace-passcode"
                  type="password"
                  autoComplete="current-password"
                  autoFocus
                  required
                  value={passcode}
                  onChange={(e) => setPasscode(e.target.value)}
                  placeholder="Enter passcode"
                  className="w-full px-3.5 py-2.5 text-sm rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] text-[#1C1917] dark:text-[#ECEEF2] focus:outline-none focus:ring-2 focus:ring-[#1D4ED8] focus:border-[#1D4ED8]"
                />
              </div>

              <button
                type="submit"
                disabled={submitting || !passcode.trim()}
                className="w-full px-4 py-2.5 text-sm font-medium rounded-md bg-[#1D4ED8] text-white hover:bg-[#1E40AF] transition-colors disabled:opacity-50 min-h-[42px]"
              >
                {submitting
                  ? 'Verifying...'
                  : authStatus.claimed
                    ? 'Unlock Workspace'
                    : 'Set Passcode & Unlock'}
              </button>
            </form>
          </div>
        )}
      </main>
    </div>
  );
}
