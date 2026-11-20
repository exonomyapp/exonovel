/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  LogOut,
  Menu,
  Moon,
  PanelLeftClose,
  PanelLeftOpen,
  RefreshCw,
  Settings,
  Sun,
} from 'lucide-react';
import { AdminSettings } from './components/AdminSettings';
import { AuthGate } from './components/AuthGate';
import { DocsSidebar } from './components/DocsSidebar';
import { MarkdownReader } from './components/MarkdownReader';
import { TableOfContents } from './components/TableOfContents';
import { getAuthInstance } from './lib/authClient';
import type { DocNode, DocPayload } from './types/docs';

interface AuthConfigResponse {
  authConfigured: boolean;
  authUrl: string | null;
  adminEmail: string;
}

interface VerifiedUser {
  authUserId: string;
  email: string | null;
  name: string | null;
  didPeer: string | null;
  authorized: boolean;
  isAdmin: boolean;
}

function flattenFiles(nodes: DocNode[]): DocNode[] {
  const result: DocNode[] = [];
  for (const node of nodes) {
    if (node.type === 'file') {
      result.push(node);
    }
    if (node.children) {
      result.push(...flattenFiles(node.children));
    }
  }
  return result;
}

function getInitialDocFromUrl(): string {
  const params = new URLSearchParams(window.location.search);
  return params.get('doc') || '';
}

export default function App() {
  const [authConfig, setAuthConfig] = useState<AuthConfigResponse | null>(null);
  const [authLoading, setAuthLoading] = useState(true);
  const [currentUser, setCurrentUser] = useState<VerifiedUser | null>(null);
  const [viewMode, setViewMode] = useState<'docs' | 'settings'>('docs');

  const [tree, setTree] = useState<DocNode[]>([]);
  const [activePath, setActivePath] = useState<string>(getInitialDocFromUrl);
  const [doc, setDoc] = useState<DocPayload | null>(null);
  const [loadingTree, setLoadingTree] = useState(false);
  const [loadingDoc, setLoadingDoc] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [mobileNavOpen, setMobileNavOpen] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState<boolean>(() => {
    try {
      return window.localStorage.getItem('exonovel-docs-sidebar-collapsed') === 'true';
    } catch {
      return false;
    }
  });
  const [activeHeadingId, setActiveHeadingId] = useState('');
  const [darkMode, setDarkMode] = useState<boolean>(() => {
    try {
      return window.localStorage.getItem('exonovel-docs-theme') === 'dark';
    } catch {
      return false;
    }
  });

  const activeDocUpdatedAtRef = useRef<string>('');

  useEffect(() => {
    const root = document.documentElement;
    if (darkMode) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
    try {
      window.localStorage.setItem('exonovel-docs-theme', darkMode ? 'dark' : 'light');
    } catch {
      // Ignore storage failures
    }
  }, [darkMode]);

  useEffect(() => {
    try {
      window.localStorage.setItem(
        'exonovel-docs-sidebar-collapsed',
        sidebarCollapsed ? 'true' : 'false'
      );
    } catch {
      // Ignore storage failures
    }
  }, [sidebarCollapsed]);

  const resolveJwt = useCallback(async (authUrl: string): Promise<string | null> => {
    const instance = getAuthInstance(authUrl);
    const cached = await instance.getJWTToken();
    if (cached) return cached;
    try {
      const res = await fetch(`${authUrl.replace(/\/$/, '')}/token`, {
        credentials: 'include',
        cache: 'no-store',
      });
      if (!res.ok) return null;
      const data = (await res.json()) as { token?: string };
      return data.token ?? null;
    } catch {
      return null;
    }
  }, []);

  const getToken = useCallback(async (): Promise<string | null> => {
    if (!authConfig?.authConfigured || !authConfig.authUrl) return null;
    try {
      return await resolveJwt(authConfig.authUrl);
    } catch {
      return null;
    }
  }, [authConfig, resolveJwt]);

  const loadSession = useCallback(async (config: AuthConfigResponse) => {
    if (!config.authConfigured || !config.authUrl) {
      setCurrentUser(null);
      setAuthLoading(false);
      return;
    }

    setAuthLoading(true);
    try {
      const instance = getAuthInstance(config.authUrl);
      const sessionRes = await instance.adapter.getSession();
      const sessionData = sessionRes?.data;
      if (!sessionData || !('user' in sessionData) || !sessionData.user) {
        setCurrentUser(null);
        setAuthLoading(false);
        return;
      }

      const jwt = await resolveJwt(config.authUrl);
      if (!jwt) {
        setCurrentUser(null);
        setAuthLoading(false);
        return;
      }

      const meRes = await fetch('/api/me', {
        cache: 'no-store',
        headers: { Authorization: `Bearer ${jwt}` },
      });

      if (!meRes.ok) {
        setCurrentUser(null);
      } else {
        const meData = (await meRes.json()) as { user: VerifiedUser };
        setCurrentUser(meData.user);
      }
    } catch {
      setCurrentUser(null);
    } finally {
      setAuthLoading(false);
    }
  }, [resolveJwt]);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    if (params.has('neon_popup') && window.opener && window.opener !== window) {
      try {
        window.opener.postMessage(
          {
            type: 'neon-auth:oauth-complete',
            verifier: params.get('neon_auth_session_verifier'),
            originalCallback: params.get('neon_popup_callback'),
          },
          window.location.origin
        );
        window.close();
      } catch {
        // Fallback to adapter initBroadcastChannel
      }
    }
  }, []);

  useEffect(() => {
    let active = true;
    fetch('/api/auth-config', { cache: 'no-store' })
      .then((res) => res.json() as Promise<AuthConfigResponse>)
      .then((cfg) => {
        if (!active) return;
        setAuthConfig(cfg);
        loadSession(cfg);
      })
      .catch(() => {
        if (!active) return;
        setAuthConfig({
          authConfigured: false,
          authUrl: null,
          adminEmail: 'exonomyapp@gmail.com',
        });
        setAuthLoading(false);
      });
    return () => {
      active = false;
    };
  }, [loadSession]);

  const handleSignOut = async () => {
    if (authConfig?.authUrl) {
      try {
        await getAuthInstance(authConfig.authUrl).adapter.signOut();
      } catch {
        // Proceed clearing local state
      }
    }
    setCurrentUser(null);
    setTree([]);
    setDoc(null);
    setViewMode('docs');
  };

  const flatFiles = useMemo(() => flattenFiles(tree), [tree]);

  const fetchTree = useCallback(
    async (silent = false) => {
      if (!currentUser?.authorized) return null;
      if (!silent) setLoadingTree(true);
      try {
        const token = await getToken();
        const response = await fetch('/api/docs', {
          cache: 'no-store',
          headers: token ? { Authorization: `Bearer ${token}` } : {},
        });
        if (!response.ok) {
          throw new Error('Failed to load documentation index.');
        }
        const data = (await response.json()) as { tree: DocNode[] };
        setTree(data.tree);
        setError(null);
        return data.tree;
      } catch (err) {
        if (!silent) {
          setError(err instanceof Error ? err.message : 'Unable to load documentation.');
        }
        return null;
      } finally {
        if (!silent) setLoadingTree(false);
      }
    },
    [currentUser?.authorized, getToken]
  );

  const fetchDoc = useCallback(
    async (docPath: string, silent = false) => {
      if (!docPath || !currentUser?.authorized) return;
      if (!silent) setLoadingDoc(true);
      try {
        const token = await getToken();
        const response = await fetch(`/api/doc?path=${encodeURIComponent(docPath)}`, {
          cache: 'no-store',
          headers: token ? { Authorization: `Bearer ${token}` } : {},
        });
        if (!response.ok) {
          throw new Error(`Could not open ${docPath}`);
        }
        const payload = (await response.json()) as DocPayload;
        if (payload.updatedAt !== activeDocUpdatedAtRef.current || !silent) {
          activeDocUpdatedAtRef.current = payload.updatedAt;
          setDoc(payload);
        }
        setError(null);
      } catch (err) {
        if (!silent) {
          setError(err instanceof Error ? err.message : 'Failed to load document.');
        }
      } finally {
        if (!silent) setLoadingDoc(false);
      }
    },
    [currentUser?.authorized, getToken]
  );

  useEffect(() => {
    if (currentUser?.authorized) {
      fetchTree(false);
    }
  }, [currentUser?.authorized, fetchTree]);

  useEffect(() => {
    if (flatFiles.length === 0) return;
    const exists = flatFiles.some((file) => file.path === activePath);
    if (!activePath || !exists) {
      const preferred =
        flatFiles.find((file) => file.path !== 'AGENTS.md') ?? flatFiles[0];
      setActivePath(preferred.path);
    }
  }, [flatFiles, activePath]);

  useEffect(() => {
    if (!activePath || !currentUser?.authorized) return;
    activeDocUpdatedAtRef.current = '';
    fetchDoc(activePath, false);

    const url = new URL(window.location.href);
    url.searchParams.set('doc', activePath);
    window.history.replaceState(null, '', url.toString());
  }, [activePath, currentUser?.authorized, fetchDoc]);

  useEffect(() => {
    if (!currentUser?.authorized) return;
    const interval = window.setInterval(() => {
      if (document.visibilityState !== 'visible') return;
      fetchTree(true);
      if (activePath) {
        fetchDoc(activePath, true);
      }
    }, 3500);
    return () => window.clearInterval(interval);
  }, [activePath, currentUser?.authorized, fetchDoc, fetchTree]);

  useEffect(() => {
    if (!doc || doc.headings.length === 0 || viewMode !== 'docs') {
      setActiveHeadingId('');
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
        if (visible.length > 0 && visible[0].target.id) {
          setActiveHeadingId(visible[0].target.id);
        }
      },
      { rootMargin: '-80px 0px -65% 0px', threshold: 0.1 }
    );

    for (const heading of doc.headings) {
      const el = document.getElementById(heading.id);
      if (el) observer.observe(el);
    }

    return () => observer.disconnect();
  }, [doc, viewMode]);

  const handleSelectDoc = (path: string) => {
    setViewMode('docs');
    setActivePath(path);
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  const handleManualRefresh = async () => {
    setRefreshing(true);
    await fetchTree(true);
    if (activePath) {
      await fetchDoc(activePath, false);
    }
    setRefreshing(false);
  };

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[#FAF8F4] dark:bg-[#111317] text-sm text-[#57534E] dark:text-[#9CA3AF]">
        Checking session...
      </div>
    );
  }

  if (!currentUser || !currentUser.authorized) {
    return (
      <AuthGate
        authConfigured={Boolean(authConfig?.authConfigured)}
        authUrl={authConfig?.authUrl ?? null}
        signedInEmail={currentUser?.email ?? null}
        didPeer={currentUser?.didPeer ?? null}
        accessDenied={Boolean(currentUser && !currentUser.authorized)}
        onAuthSuccess={() => {
          if (authConfig) loadSession(authConfig);
        }}
        onSignOut={handleSignOut}
      />
    );
  }

  const currentIndex = flatFiles.findIndex((f) => f.path === activePath);
  const prevDoc = currentIndex > 0 ? flatFiles[currentIndex - 1] : null;
  const nextDoc =
    currentIndex >= 0 && currentIndex < flatFiles.length - 1
      ? flatFiles[currentIndex + 1]
      : null;

  const breadcrumbs =
    viewMode === 'settings'
      ? ['Admin Settings']
      : activePath
        ? activePath.split('/')
        : [];

  return (
    <div className="min-h-screen flex bg-[#FAF8F4] dark:bg-[#111317] text-[#1C1917] dark:text-[#ECEEF2]">
      <DocsSidebar
        tree={tree}
        activePath={activePath}
        onSelectDoc={handleSelectDoc}
        collapsed={sidebarCollapsed}
        mobileOpen={mobileNavOpen}
        onCloseMobile={() => setMobileNavOpen(false)}
      />

      <div className="flex-1 min-w-0 flex flex-col">
        <header className="sticky top-0 z-30 h-14 px-4 sm:px-8 border-b border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4]/95 dark:bg-[#111317]/95 backdrop-blur-sm flex items-center justify-between gap-4">
          <div className="flex items-center gap-3 min-w-0">
            <button
              type="button"
              onClick={() => setMobileNavOpen(true)}
              className="lg:hidden p-2 -ml-2 rounded-md text-[#44403C] dark:text-[#D1D5DB] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] min-h-[40px] min-w-[40px] flex items-center justify-center shrink-0"
              aria-label="Open documentation explorer"
            >
              <Menu className="w-5 h-5" />
            </button>

            <button
              type="button"
              onClick={() => setSidebarCollapsed((prev) => !prev)}
              className="hidden lg:inline-flex items-center justify-center p-2 -ml-2 rounded-md text-[#44403C] dark:text-[#D1D5DB] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] min-h-[40px] min-w-[40px] shrink-0 transition-colors"
              aria-label={sidebarCollapsed ? 'Expand navigation sidebar' : 'Collapse navigation sidebar'}
              aria-expanded={!sidebarCollapsed}
              title={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            >
              {sidebarCollapsed ? (
                <PanelLeftOpen className="w-4 h-4" />
              ) : (
                <PanelLeftClose className="w-4 h-4" />
              )}
            </button>

            <nav
              aria-label="Breadcrumb"
              className="flex items-center gap-1.5 text-xs sm:text-sm text-[#57534E] dark:text-[#9CA3AF] min-w-0"
            >
              <span className="font-medium text-[#1C1917] dark:text-[#ECEEF2] whitespace-nowrap shrink-0">
                Docs
              </span>
              {breadcrumbs.map((segment, index) => {
                const isLast = index === breadcrumbs.length - 1;
                return (
                  <span key={`${segment}-${index}`} className="flex items-center gap-1.5 min-w-0">
                    <span aria-hidden="true" className="opacity-50 shrink-0">
                      /
                    </span>
                    <span
                      className={`truncate ${
                        isLast
                          ? 'text-[#1C1917] dark:text-[#ECEEF2] font-medium'
                          : 'hidden sm:inline'
                      }`}
                    >
                      {segment}
                    </span>
                  </span>
                );
              })}
            </nav>
          </div>

          <div className="flex items-center gap-2 shrink-0">
            {viewMode === 'docs' && (
              <button
                type="button"
                onClick={handleManualRefresh}
                disabled={refreshing}
                className="inline-flex items-center gap-2 px-3 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] text-[#1C1917] dark:text-[#ECEEF2] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors whitespace-nowrap shrink-0 min-h-[40px]"
              >
                <RefreshCw className={`w-3.5 h-3.5 ${refreshing ? 'animate-spin' : ''}`} />
                <span className="hidden sm:inline">Refresh</span>
              </button>
            )}

            {currentUser.isAdmin && (
              <button
                type="button"
                onClick={() =>
                  setViewMode((prev) => (prev === 'settings' ? 'docs' : 'settings'))
                }
                className={`inline-flex items-center gap-2 px-3 py-2 text-xs font-medium rounded-md border transition-colors whitespace-nowrap shrink-0 min-h-[40px] ${
                  viewMode === 'settings'
                    ? 'border-[#1D4ED8] text-[#1D4ED8] dark:border-[#60A5FA] dark:text-[#60A5FA] bg-[#EFF6FF] dark:bg-[#172554]/40'
                    : 'border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] text-[#1C1917] dark:text-[#ECEEF2] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D]'
                }`}
              >
                <Settings className="w-3.5 h-3.5" />
                <span className="hidden sm:inline">Settings</span>
              </button>
            )}

            <button
              type="button"
              onClick={() => setDarkMode((prev) => !prev)}
              className="inline-flex items-center gap-2 px-3 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] text-[#1C1917] dark:text-[#ECEEF2] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors whitespace-nowrap shrink-0 min-h-[40px]"
              aria-label={darkMode ? 'Switch to light theme' : 'Switch to dark theme'}
            >
              {darkMode ? <Sun className="w-3.5 h-3.5" /> : <Moon className="w-3.5 h-3.5" />}
              <span className="hidden sm:inline">{darkMode ? 'Light' : 'Dark'}</span>
            </button>

            <button
              type="button"
              onClick={handleSignOut}
              className="inline-flex items-center gap-2 px-3 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] text-[#1C1917] dark:text-[#ECEEF2] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors whitespace-nowrap shrink-0 min-h-[40px]"
              aria-label="Sign out"
            >
              <LogOut className="w-3.5 h-3.5" />
              <span className="hidden md:inline">Sign out</span>
            </button>
          </div>
        </header>

        <main className="flex-1 px-5 sm:px-8 lg:px-10 py-8 sm:py-10">
          {viewMode === 'settings' && currentUser.isAdmin ? (
            <AdminSettings
              getToken={getToken}
              activeDocPath={activePath}
              currentUserDid={currentUser.didPeer}
              currentUserEmail={currentUser.email}
              onBackToDocs={() => setViewMode('docs')}
            />
          ) : error ? (
            <div className="max-w-xl rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/70 dark:bg-[#171A21] p-6">
              <h2 className="font-display text-lg font-semibold text-[#1C1917] dark:text-[#ECEEF2]">
                Unable to load document
              </h2>
              <p className="mt-2 text-sm text-[#57534E] dark:text-[#9CA3AF]">{error}</p>
              <button
                type="button"
                onClick={handleManualRefresh}
                className="mt-4 px-4 py-2 text-xs font-medium rounded-md bg-[#1D4ED8] text-white hover:bg-[#1E40AF] transition-colors whitespace-nowrap shrink-0 min-h-[40px]"
              >
                Retry
              </button>
            </div>
          ) : loadingTree || (loadingDoc && !doc) ? (
            <div className="w-full space-y-4 animate-pulse">
              <div className="h-4 w-48 rounded bg-[#E5E0D5] dark:bg-[#262B36]" />
              <div className="h-10 w-3/4 rounded bg-[#E5E0D5] dark:bg-[#262B36]" />
              <div className="h-4 w-full rounded bg-[#E5E0D5] dark:bg-[#262B36] mt-6" />
              <div className="h-4 w-5/6 rounded bg-[#E5E0D5] dark:bg-[#262B36]" />
              <div className="h-4 w-2/3 rounded bg-[#E5E0D5] dark:bg-[#262B36]" />
            </div>
          ) : doc ? (
            <div className="w-full flex items-start gap-10">
              <MarkdownReader
                doc={doc}
                prevDoc={prevDoc}
                nextDoc={nextDoc}
                onNavigateDoc={handleSelectDoc}
              />
              <TableOfContents headings={doc.headings} activeId={activeHeadingId} />
            </div>
          ) : (
            <div className="max-w-xl rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/60 dark:bg-[#171A21] p-6">
              <h2 className="font-display text-lg font-semibold text-[#1C1917] dark:text-[#ECEEF2]">
                No documentation files yet
              </h2>
              <p className="mt-2 text-sm text-[#57534E] dark:text-[#9CA3AF]">
                Select or create a Markdown document to read it here.
              </p>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
