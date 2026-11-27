import { useCallback, useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import {
  Download,
  Eye,
  EyeOff,
  ExternalLink,
  GitBranch,
  GitMerge,
  GitPullRequest,
  Plus,
  Search,
  Trash2,
  Upload,
  X,
} from 'lucide-react';

interface GitHubOpenPull {
  number: number;
  title: string;
  state: string;
  htmlUrl: string;
  headRef: string;
  baseRef: string;
  userLogin: string;
  createdAt: string;
}

interface GitHubCheckRunSummary {
  id: number;
  name: string;
  status: string;
  conclusion: string | null;
  htmlUrl: string;
  startedAt: string | null;
  completedAt: string | null;
}

interface GitHubSyncStatus {
  configured: boolean;
  repo: string;
  baseBranch: string;
  tokenUser: string | null;
  scopes: string[] | null;
  permissions: {
    admin: boolean;
    maintain: boolean;
    push: boolean;
    triage: boolean;
    pull: boolean;
  } | null;
  headSha: string | null;
  headMessage: string | null;
  protectedBranch: boolean | null;
  openPulls: GitHubOpenPull[];
  checkRuns: GitHubCheckRunSummary[];
  error: string | null;
}

type SyncFileCategory = 'app' | 'console' | 'superseded';

interface WorkspaceDiffEntry {
  path: string;
  status: 'modified' | 'added_local' | 'remote_only';
  category: SyncFileCategory;
  sizeBytes: number;
  remoteSizeBytes?: number;
  remoteSha?: string;
  hasBase: boolean;
  remoteUpdatedSinceBase: boolean;
  canPush: boolean;
  canPull: boolean;
  canDeleteRemote: boolean;
}

interface GitignoreRuleEntry {
  pattern: string;
  locked: boolean;
  matchedCount: number;
}

interface IgnoredFileEntry {
  path: string;
  matchedRule: string;
  ruleLocked: boolean;
  existsLocal: boolean;
  existsRemote: boolean;
  sizeBytes: number;
  remoteSha?: string;
}

interface WorkspaceFileDiffDetail {
  path: string;
  existsLocal: boolean;
  existsRemote: boolean;
  remoteSha: string | null;
  hasBase: boolean;
  baseSha: string | null;
  remoteUpdatedSinceBase: boolean;
  isBinary: boolean;
  localSize: number;
  remoteSize: number;
  localContent: string | null;
  remoteContent: string | null;
  baseContent: string | null;
  truncated: boolean;
}

interface MergeConflictEntry {
  path: string;
  direction: 'pull' | 'push';
  reason: 'overlap' | 'remote_ahead_clean' | 'no_base' | 'binary';
  remoteSha: string;
  baseSha: string | null;
  isBinary: boolean;
  baseContent: string | null;
  localContent: string | null;
  remoteContent: string | null;
  suggestedContent: string | null;
}

interface GitHubSyncPanelProps {
  getToken: () => Promise<string | null>;
  refreshTrigger?: number;
  onLoadingChange?: (loading: boolean) => void;
  onUnauthorized?: () => void;
}

type CategoryFilter = 'all' | 'app' | 'console' | 'superseded' | 'ignored';

function buildSuggestedCommit(
  pushFiles: WorkspaceDiffEntry[],
  deletePaths: string[]
): string {
  const total = pushFiles.length + deletePaths.length;
  if (total === 0) return '';
  if (total === 1) {
    if (deletePaths.length === 1) {
      return `chore: remove ${deletePaths[0]}`;
    }
    return `chore: update ${pushFiles[0].path}`;
  }
  if (pushFiles.length > 0 && deletePaths.length > 0) {
    return `chore: update ${pushFiles.length} file(s) and remove ${deletePaths.length} file(s)`;
  }
  if (deletePaths.length > 0) {
    return `chore: remove ${deletePaths.length} file(s) from main`;
  }
  return `chore: sync ${pushFiles.length} file(s) to exonomyapp/exonovel`;
}

function statusDisplayLabel(status: WorkspaceDiffEntry['status']): string {
  if (status === 'added_local') return 'new locally';
  if (status === 'remote_only') return 'on main only';
  return 'modified';
}

function statusColorClass(status: WorkspaceDiffEntry['status']): string {
  if (status === 'added_local') {
    return 'text-emerald-700 dark:text-emerald-400';
  }
  if (status === 'remote_only') {
    return 'text-[#1D4ED8] dark:text-[#60A5FA]';
  }
  return 'text-amber-700 dark:text-amber-400';
}

function categoryDisplayLabel(category: SyncFileCategory): string {
  if (category === 'superseded') return 'superseded';
  if (category === 'console') return 'console';
  return 'app';
}

export function GitHubSyncPanel({
  getToken,
  refreshTrigger = 0,
  onLoadingChange,
  onUnauthorized,
}: GitHubSyncPanelProps) {
  const [githubStatus, setGithubStatus] = useState<GitHubSyncStatus | null>(null);
  const [diffHeadSha, setDiffHeadSha] = useState<string | null>(null);
  const [githubDiff, setGithubDiff] = useState<WorkspaceDiffEntry[]>([]);
  const [gitignoreRules, setGitignoreRules] = useState<GitignoreRuleEntry[]>([]);
  const [ignoredFiles, setIgnoredFiles] = useState<IgnoredFileEntry[]>([]);
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [selectedUntrackPaths, setSelectedUntrackPaths] = useState<string[]>([]);
  const [categoryFilter, setCategoryFilter] = useState<CategoryFilter>('all');
  const [selectionScope, setSelectionScope] = useState<
    'none' | 'all' | 'pushable' | 'pullable' | 'superseded' | 'custom'
  >('none');
  const [filterBySelectedScope, setFilterBySelectedScope] = useState(false);
  const [includeRemoteDeletions, setIncludeRemoteDeletions] = useState(false);

  const [newIgnorePattern, setNewIgnorePattern] = useState('');
  const [fileFilterInput, setFileFilterInput] = useState('');
  const [debouncedFileFilter, setDebouncedFileFilter] = useState('');
  const [gitignoreBusy, setGitignoreBusy] = useState(false);

  const [githubLoading, setGithubLoading] = useState(false);
  const [githubSyncing, setGithubSyncing] = useState(false);
  const [githubPulling, setGithubPulling] = useState(false);
  const [mergingPr, setMergingPr] = useState<number | null>(null);
  const [confirmDirectPush, setConfirmDirectPush] = useState(false);

  const [mergeQueue, setMergeQueue] = useState<MergeConflictEntry[]>([]);
  const [mergeDrafts, setMergeDrafts] = useState<Record<string, string>>({});
  const [resolvingMergePath, setResolvingMergePath] = useState<string | null>(null);

  const [previewPath, setPreviewPath] = useState<string | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewData, setPreviewData] = useState<WorkspaceFileDiffDetail | null>(null);

  const [commitMessage, setCommitMessage] = useState('');
  const [branchName, setBranchName] = useState(
    () => `sync/exonovel-${new Date().toISOString().slice(0, 10)}`
  );
  const [prTitle, setPrTitle] = useState('');
  const [githubNotice, setGithubNotice] = useState<{
    message: string;
    url?: string;
    label?: string;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);

  const getTokenRef = useRef(getToken);
  const onLoadingChangeRef = useRef(onLoadingChange);
  const onUnauthorizedRef = useRef(onUnauthorized);
  const fetchInFlightRef = useRef(false);

  useEffect(() => {
    getTokenRef.current = getToken;
  }, [getToken]);

  useEffect(() => {
    onLoadingChangeRef.current = onLoadingChange;
  }, [onLoadingChange]);

  useEffect(() => {
    onUnauthorizedRef.current = onUnauthorized;
  }, [onUnauthorized]);

  const authFetch = useCallback(async (url: string, init?: RequestInit) => {
    const token = await getTokenRef.current();
    const res = await fetch(url, {
      ...init,
      credentials: 'include',
      headers: {
        'Content-Type': 'application/json',
        'X-Requested-With': 'XMLHttpRequest',
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(init?.headers || {}),
      },
    });
    if (res.status === 401 && onUnauthorizedRef.current) {
      onUnauthorizedRef.current();
    }
    return res;
  }, []);

  const loadGitHubSync = useCallback(async () => {
    if (fetchInFlightRef.current) return;
    fetchInFlightRef.current = true;
    setGithubLoading(true);
    setError(null);
    setConfirmDirectPush(false);
    try {
      const [statusRes, diffRes] = await Promise.all([
        authFetch('/api/admin/github/status'),
        authFetch('/api/admin/github/diff'),
      ]);

      if (statusRes.ok) {
        const statusData = (await statusRes.json()) as GitHubSyncStatus;
        setGithubStatus(statusData);
      } else {
        const errData = await statusRes.json().catch(() => ({}));
        setError(errData.error || 'Unable to check GitHub status.');
      }

      if (diffRes.ok) {
        const diffData = (await diffRes.json()) as {
          headSha: string;
          files: WorkspaceDiffEntry[];
          gitignore?: {
            rules: GitignoreRuleEntry[];
            ignoredFiles: IgnoredFileEntry[];
          };
        };
        setDiffHeadSha(diffData.headSha || null);
        setGithubDiff(diffData.files);
        setGitignoreRules(diffData.gitignore?.rules || []);
        setIgnoredFiles(diffData.gitignore?.ignoredFiles || []);

        const validDiffPaths = new Set(diffData.files.map((f) => f.path));
        setSelectedPaths((prev) => {
          const preserved = prev.filter((p) => validDiffPaths.has(p));
          if (preserved.length > 0) return preserved;
          return diffData.files.filter((f) => f.category === 'app').map((f) => f.path);
        });

        const validRemoteIgnored = new Set(
          (diffData.gitignore?.ignoredFiles || [])
            .filter((f) => f.existsRemote)
            .map((f) => f.path)
        );
        setSelectedUntrackPaths((prev) => prev.filter((p) => validRemoteIgnored.has(p)));
      } else {
        setDiffHeadSha(null);
        setGithubDiff([]);
        setGitignoreRules([]);
        setIgnoredFiles([]);
        setSelectedPaths([]);
        setSelectedUntrackPaths([]);
      }
    } catch {
      setError('Unable to reach GitHub sync service.');
    } finally {
      fetchInFlightRef.current = false;
      setGithubLoading(false);
    }
  }, [authFetch]);

  useEffect(() => {
    onLoadingChangeRef.current?.(githubLoading);
  }, [githubLoading]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedFileFilter(fileFilterInput.trim().toLowerCase());
    }, 180);
    return () => window.clearTimeout(timer);
  }, [fileFilterInput]);

  useEffect(() => {
    loadGitHubSync();
  }, [loadGitHubSync, refreshTrigger]);

  const countsByCategory = useMemo(() => {
    let app = 0;
    let consoleCount = 0;
    let superseded = 0;
    for (const file of githubDiff) {
      if (file.category === 'app') app += 1;
      else if (file.category === 'console') consoleCount += 1;
      else superseded += 1;
    }
    return {
      all: githubDiff.length,
      app,
      console: consoleCount,
      superseded,
      ignored: ignoredFiles.length,
    };
  }, [githubDiff, ignoredFiles.length]);

  const isSelectAllSelected = selectionScope === 'all';

  const baseCategoryAndTextFiles = useMemo(() => {
    if (categoryFilter === 'ignored') return [];
    const byCategory =
      categoryFilter === 'all'
        ? githubDiff
        : githubDiff.filter((f) => f.category === categoryFilter);
    if (!debouncedFileFilter) return byCategory;
    return byCategory.filter((f) =>
      f.path.toLowerCase().includes(debouncedFileFilter)
    );
  }, [githubDiff, categoryFilter, debouncedFileFilter]);

  const visibleFiles = useMemo(() => {
    if (!filterBySelectedScope || isSelectAllSelected) {
      return baseCategoryAndTextFiles;
    }
    if (selectionScope === 'pushable') {
      return baseCategoryAndTextFiles.filter((f) => f.canPush);
    }
    if (selectionScope === 'pullable') {
      return baseCategoryAndTextFiles.filter((f) => f.canPull);
    }
    if (selectionScope === 'superseded') {
      return baseCategoryAndTextFiles.filter((f) => f.category === 'superseded');
    }
    const selectedSet = new Set(selectedPaths);
    return baseCategoryAndTextFiles.filter((f) => selectedSet.has(f.path));
  }, [
    baseCategoryAndTextFiles,
    filterBySelectedScope,
    isSelectAllSelected,
    selectionScope,
    selectedPaths,
  ]);

  const visibleIgnoredFiles = useMemo(() => {
    if (!debouncedFileFilter) return ignoredFiles;
    return ignoredFiles.filter((f) =>
      f.path.toLowerCase().includes(debouncedFileFilter)
    );
  }, [ignoredFiles, debouncedFileFilter]);

  const selectedEntries = useMemo(() => {
    const selectedSet = new Set(selectedPaths);
    return githubDiff.filter((f) => selectedSet.has(f.path));
  }, [githubDiff, selectedPaths]);

  const selectedPushFiles = useMemo(
    () => selectedEntries.filter((f) => f.canPush),
    [selectedEntries]
  );

  const selectedPullFiles = useMemo(
    () => selectedEntries.filter((f) => f.canPull),
    [selectedEntries]
  );

  const selectedPullModifiedCount = useMemo(
    () => selectedPullFiles.filter((f) => f.status === 'modified').length,
    [selectedPullFiles]
  );

  const selectedRemoteDeleteCandidates = useMemo(
    () => selectedEntries.filter((f) => f.canDeleteRemote),
    [selectedEntries]
  );

  const allDeletePaths = useMemo(() => {
    const out = new Set<string>(selectedUntrackPaths);
    if (includeRemoteDeletions) {
      for (const f of selectedRemoteDeleteCandidates) {
        out.add(f.path);
      }
    }
    return Array.from(out);
  }, [includeRemoteDeletions, selectedRemoteDeleteCandidates, selectedUntrackPaths]);

  const effectivePushCount = selectedPushFiles.length + allDeletePaths.length;
  const suggestedCommit = buildSuggestedCommit(selectedPushFiles, allDeletePaths);

  const togglePathSelection = (filePath: string) => {
    setConfirmDirectPush(false);
    setSelectionScope('custom');
    setSelectedPaths((prev) =>
      prev.includes(filePath)
        ? prev.filter((p) => p !== filePath)
        : [...prev, filePath]
    );
  };

  const toggleUntrackSelection = (filePath: string) => {
    setConfirmDirectPush(false);
    setSelectedUntrackPaths((prev) =>
      prev.includes(filePath)
        ? prev.filter((p) => p !== filePath)
        : [...prev, filePath]
    );
  };

  const handleSelectScope = (
    scope: 'all' | 'pushable' | 'pullable',
    predicate: (entry: WorkspaceDiffEntry) => boolean
  ) => {
    setConfirmDirectPush(false);
    setSelectionScope(scope);
    if (scope === 'all') {
      setFilterBySelectedScope(false);
    }
    const matching = baseCategoryAndTextFiles.filter(predicate).map((f) => f.path);
    setSelectedPaths(matching);
  };

  const handleGitignoreMutation = async (
    action: 'add' | 'remove' | 'unignore',
    value: string
  ) => {
    const cleaned = value.trim();
    if (!cleaned) return;
    setGitignoreBusy(true);
    setError(null);
    setGithubNotice(null);
    try {
      const res = await authFetch('/api/admin/github/gitignore', {
        method: 'POST',
        body: JSON.stringify({ action, value: cleaned }),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        throw new Error(data.error || 'Unable to update .gitignore.');
      }
      if (action === 'add') {
        setNewIgnorePattern('');
        setGithubNotice({
          message: `Added "${data.pattern}" to .gitignore.`,
        });
      } else if (action === 'remove') {
        setGithubNotice({
          message: `Removed "${data.pattern}" from .gitignore.`,
        });
      } else {
        setGithubNotice({
          message:
            data.action === 'added_negation'
              ? `Un-ignored file with "${data.pattern}" in .gitignore.`
              : `Removed "${data.pattern}" from .gitignore.`,
        });
      }
      await loadGitHubSync();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to update .gitignore.');
    } finally {
      setGitignoreBusy(false);
    }
  };

  const handleAddPatternSubmit = async (e: FormEvent) => {
    e.preventDefault();
    await handleGitignoreMutation('add', newIgnorePattern);
  };

  const handlePreviewFile = async (filePath: string) => {
    if (previewPath === filePath && previewData) {
      setPreviewPath(null);
      setPreviewData(null);
      return;
    }
    setPreviewPath(filePath);
    setPreviewLoading(true);
    setPreviewData(null);
    setError(null);
    try {
      const res = await authFetch(
        `/api/admin/github/file-diff?path=${encodeURIComponent(filePath)}`
      );
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        throw new Error(data.error || 'Unable to inspect file diff.');
      }
      setPreviewData(data as WorkspaceFileDiffDetail);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to inspect file diff.');
      setPreviewPath(null);
    } finally {
      setPreviewLoading(false);
    }
  };

  const openConflictsInResolver = (conflicts: MergeConflictEntry[]) => {
    setMergeQueue(conflicts);
    setMergeDrafts(() => {
      const next: Record<string, string> = {};
      for (const c of conflicts) {
        if (!c.isBinary) {
          next[c.path] = c.suggestedContent ?? c.localContent ?? c.remoteContent ?? '';
        }
      }
      return next;
    });
  };

  const handlePullFromMain = async (explicitOverwritePaths?: string[]) => {
    const overwriteSet = new Set(explicitOverwritePaths || []);
    const targetPaths =
      overwriteSet.size > 0
        ? Array.from(overwriteSet)
        : selectedPullFiles.map((f) => f.path);

    if (targetPaths.length === 0) {
      setError('Select at least one "modified" or "on main only" file to pull.');
      return;
    }

    setGithubPulling(true);
    setError(null);
    setGithubNotice(null);
    try {
      const res = await authFetch('/api/admin/github/pull', {
        method: 'POST',
        body: JSON.stringify({
          paths: targetPaths,
          overwritePaths: overwriteSet.size > 0 ? targetPaths : undefined,
          expectedHeadSha: diffHeadSha || undefined,
        }),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        if (res.status === 409 && Array.isArray(data.conflicts) && data.conflicts.length > 0) {
          openConflictsInResolver(data.conflicts as MergeConflictEntry[]);
        }
        throw new Error(data.error || 'Pull from main failed.');
      }

      const conflicts = Array.isArray(data.conflicts)
        ? (data.conflicts as MergeConflictEntry[])
        : [];
      const autoMergedCount = Array.isArray(data.autoMergedPaths)
        ? data.autoMergedPaths.length
        : 0;
      const keptCount = Array.isArray(data.keptLocalPaths)
        ? data.keptLocalPaths.length
        : 0;

      if (conflicts.length > 0) {
        openConflictsInResolver(conflicts);
      } else if (overwriteSet.size > 0) {
        setMergeQueue((prev) => prev.filter((c) => !overwriteSet.has(c.path)));
      }

      const parts: string[] = [];
      if (data.filesPulled > 0) {
        const mergeNote =
          autoMergedCount > 0 ? ` (${autoMergedCount} 3-way merged with local edits)` : '';
        parts.push(
          `Synced ${data.filesPulled} file(s) from exonomyapp/exonovel@main (${String(data.headSha || '').slice(0, 8)})${mergeNote}`
        );
      }
      if (keptCount > 0) {
        parts.push(`${keptCount} unchanged remotely (local edits preserved)`);
      }
      if (conflicts.length > 0) {
        parts.push(`${conflicts.length} file(s) need merge review below`);
      }

      if (parts.length > 0) {
        setGithubNotice({ message: `${parts.join(' · ')}.` });
      }

      setPreviewPath(null);
      setPreviewData(null);
      await loadGitHubSync();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Pull from main failed.');
    } finally {
      setGithubPulling(false);
    }
  };

  const handleResolveMerge = async (
    conflict: MergeConflictEntry,
    contentOverride?: string
  ) => {
    const resolvedContent =
      contentOverride !== undefined ? contentOverride : mergeDrafts[conflict.path];
    if (typeof resolvedContent !== 'string') {
      setError('Merged content is required before applying.');
      return;
    }

    setResolvingMergePath(conflict.path);
    setError(null);
    try {
      const res = await authFetch('/api/admin/github/resolve-merge', {
        method: 'POST',
        body: JSON.stringify({
          path: conflict.path,
          resolvedContent,
          remoteSha: conflict.remoteSha,
        }),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        throw new Error(data.error || 'Unable to save merged file.');
      }

      setMergeQueue((prev) => prev.filter((item) => item.path !== conflict.path));
      setGithubNotice({
        message: `Saved merged ${conflict.path} against GitHub main baseline (${conflict.remoteSha.slice(0, 8)}).`,
      });
      setPreviewPath(null);
      setPreviewData(null);
      await loadGitHubSync();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to save merged file.');
    } finally {
      setResolvingMergePath(null);
    }
  };

  const handleGitHubSync = async (mode: 'pr' | 'direct') => {
    if (effectivePushCount === 0) {
      setError('Select at least one local file to push or remove.');
      return;
    }
    const effectiveCommit = commitMessage.trim() || suggestedCommit;
    const knownRemoteShas: Record<string, string> = {};
    for (const file of selectedPushFiles) {
      if (file.remoteSha) {
        knownRemoteShas[file.path] = file.remoteSha;
      }
    }

    setGithubSyncing(true);
    setError(null);
    setGithubNotice(null);
    setConfirmDirectPush(false);
    try {
      const res = await authFetch('/api/admin/github/sync', {
        method: 'POST',
        body: JSON.stringify({
          mode,
          paths: selectedPushFiles.map((f) => f.path),
          deletePaths: allDeletePaths,
          commitMessage: effectiveCommit,
          branchName: mode === 'pr' ? branchName : undefined,
          prTitle: mode === 'pr' ? prTitle.trim() || effectiveCommit : undefined,
          expectedHeadSha: diffHeadSha || undefined,
          knownRemoteShas,
        }),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        if (res.status === 409 && Array.isArray(data.conflicts) && data.conflicts.length > 0) {
          openConflictsInResolver(data.conflicts as MergeConflictEntry[]);
        }
        throw new Error(data.error || 'GitHub sync failed.');
      }
      const deletedNote =
        data.filesDeleted > 0 ? `, removed ${data.filesDeleted} file(s)` : '';
      if (mode === 'pr') {
        setGithubNotice({
          message: `Opened Pull Request #${data.pullNumber} from ${data.branchName} (${data.filesSynced} pushed${deletedNote}).`,
          url: data.pullUrl,
          label: `View PR #${data.pullNumber}`,
        });
      } else {
        setGithubNotice({
          message: `Committed to main (${String(data.commitSha || '').slice(0, 8)}): ${data.filesSynced} pushed${deletedNote}.`,
          url: data.commitUrl,
          label: 'View Commit',
        });
      }
      setCommitMessage('');
      setPrTitle('');
      setIncludeRemoteDeletions(false);
      setSelectedUntrackPaths([]);
      setMergeQueue([]);
      setPreviewPath(null);
      setPreviewData(null);
      await loadGitHubSync();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'GitHub sync failed.');
    } finally {
      setGithubSyncing(false);
    }
  };

  const handleMergePullRequest = async (pullNumber: number) => {
    setMergingPr(pullNumber);
    setError(null);
    setGithubNotice(null);
    try {
      const res = await authFetch('/api/admin/github/merge', {
        method: 'POST',
        body: JSON.stringify({ pullNumber, mergeMethod: 'merge' }),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        throw new Error(data.error || `Unable to merge PR #${pullNumber}.`);
      }
      setGithubNotice({
        message: `Merged Pull Request #${pullNumber} into main.`,
      });
      await loadGitHubSync();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to merge pull request.');
    } finally {
      setMergingPr(null);
    }
  };

  const filterTabs: Array<{ id: CategoryFilter; label: string; count: number }> = [
    { id: 'all', label: 'All Differences', count: countsByCategory.all },
    { id: 'app', label: 'ExoNovel App & CI', count: countsByCategory.app },
    { id: 'console', label: 'Sync Console', count: countsByCategory.console },
    { id: 'superseded', label: 'Superseded on main', count: countsByCategory.superseded },
    { id: 'ignored', label: 'Ignored (.gitignore)', count: countsByCategory.ignored },
  ];

  return (
    <div>
      <div>
        <h1 className="font-display text-2xl font-semibold tracking-tight">
          Sync Console
        </h1>
        <p className="mt-1 text-sm text-[#57534E] dark:text-[#9CA3AF]">
          Inspect differences against{' '}
          <code className="font-mono text-xs">exonomyapp/exonovel@main</code>, manage{' '}
          <code className="font-mono text-xs">.gitignore</code> rules, selectively pull updates down, or push selected changes.
        </p>
      </div>

      {error && (
        <div className="mt-4 p-3.5 rounded-lg border border-red-300 dark:border-red-900/60 bg-red-50 dark:bg-red-950/40 text-xs text-red-700 dark:text-red-300">
          {error}
        </div>
      )}

      {githubNotice && (
        <div className="mt-4 flex flex-wrap items-center justify-between gap-3 p-3.5 rounded-lg border border-emerald-300 dark:border-emerald-900/60 bg-emerald-50 dark:bg-emerald-950/30 text-xs text-emerald-800 dark:text-emerald-200">
          <span>{githubNotice.message}</span>
          {githubNotice.url && (
            <a
              href={githubNotice.url}
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-1 font-medium underline whitespace-nowrap shrink-0"
            >
              <span>{githubNotice.label || 'Open on GitHub'}</span>
              <ExternalLink className="w-3 h-3" />
            </a>
          )}
        </div>
      )}

      <div className="mt-5 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] p-5 sm:p-6 space-y-6">
        {!githubStatus ? (
          <div className="text-xs text-[#57534E] dark:text-[#9CA3AF]">
            Checking GitHub repository status...
          </div>
        ) : githubStatus.error ? (
          <div className="text-xs text-red-700 dark:text-red-300">
            {githubStatus.error}
          </div>
        ) : (
          <>
            <div className="grid gap-4 sm:grid-cols-3 text-xs">
              <div className="border-b sm:border-b-0 sm:border-r border-[#E5E0D5] dark:border-[#262B36] pb-3 sm:pb-0 sm:pr-4">
                <div className="text-[#57534E] dark:text-[#9CA3AF]">Authenticated Account</div>
                <div className="mt-1 font-medium font-mono">
                  @{githubStatus.tokenUser || 'unknown'}
                </div>
                <div className="mt-0.5 text-[#57534E] dark:text-[#9CA3AF]">
                  Write access: {githubStatus.permissions?.push ? 'enabled' : 'read-only'}
                  {githubStatus.scopes ? ` · ${githubStatus.scopes.join(', ')}` : ''}
                </div>
              </div>

              <div className="border-b sm:border-b-0 sm:border-r border-[#E5E0D5] dark:border-[#262B36] pb-3 sm:pb-0 sm:pr-4">
                <div className="text-[#57534E] dark:text-[#9CA3AF]">Target Branch</div>
                <div className="mt-1 font-medium font-mono tabular-nums">
                  {githubStatus.baseBranch} ({githubStatus.headSha?.slice(0, 8) || '—'})
                </div>
                <div className="mt-0.5 truncate text-[#57534E] dark:text-[#9CA3AF]">
                  {githubStatus.headMessage || 'No commit message'}
                </div>
              </div>

              <div>
                <div className="text-[#57534E] dark:text-[#9CA3AF]">Branch Protection</div>
                <div className="mt-1 font-medium">
                  {githubStatus.protectedBranch ? 'Protected' : 'Direct push allowed'}
                </div>
                <div className="mt-0.5 text-[#57534E] dark:text-[#9CA3AF] tabular-nums">
                  Open PRs: {githubStatus.openPulls.length}
                </div>
              </div>
            </div>

            {githubStatus.checkRuns.length > 0 && (
              <div className="pt-4 border-t border-[#E5E0D5] dark:border-[#262B36]">
                <div className="text-xs font-medium mb-2">
                  Latest GitHub Actions on <code className="font-mono">main</code>
                </div>
                <div className="divide-y divide-[#E5E0D5] dark:divide-[#262B36] border border-[#E5E0D5] dark:border-[#262B36] rounded-md bg-[#FAF8F4] dark:bg-[#111317]">
                  {githubStatus.checkRuns.map((run) => {
                    const stateLabel = run.conclusion
                      ? `${run.status} · ${run.conclusion}`
                      : run.status.replace('_', ' ');
                    const stateColor =
                      run.conclusion === 'success'
                        ? 'text-emerald-700 dark:text-emerald-400'
                        : run.conclusion === 'failure' || run.conclusion === 'cancelled'
                          ? 'text-red-700 dark:text-red-400'
                          : 'text-amber-700 dark:text-amber-400';
                    return (
                      <div
                        key={run.id}
                        className="flex flex-wrap items-center justify-between gap-3 px-3.5 py-2.5 text-xs"
                      >
                        <span className="font-medium">{run.name}</span>
                        <div className="flex items-center gap-3 shrink-0">
                          <span className={`font-mono ${stateColor}`}>{stateLabel}</span>
                          <a
                            href={run.htmlUrl}
                            target="_blank"
                            rel="noreferrer"
                            className="inline-flex items-center gap-1 text-[#1D4ED8] dark:text-[#60A5FA] hover:underline whitespace-nowrap"
                          >
                            <span>View Run</span>
                            <ExternalLink className="w-3 h-3" />
                          </a>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}

            <div className="pt-4 border-t border-[#E5E0D5] dark:border-[#262B36] space-y-3">
              <div className="flex flex-wrap items-center gap-1.5">
                {filterTabs.map((tab) => {
                  const active = categoryFilter === tab.id;
                  return (
                    <button
                      key={tab.id}
                      type="button"
                      onClick={() => {
                        setCategoryFilter(tab.id);
                        setSelectionScope((prev) => (prev === 'all' ? 'custom' : prev));
                      }}
                      className={`px-3 py-1.5 text-xs rounded-md border transition-colors whitespace-nowrap tabular-nums ${
                        active
                          ? 'border-[#1D4ED8] bg-[#1D4ED8]/10 text-[#1D4ED8] dark:text-[#60A5FA] font-medium'
                          : 'border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2]'
                      }`}
                    >
                      {tab.label} ({tab.count})
                    </button>
                  );
                })}
              </div>

              {(githubDiff.length > 0 || ignoredFiles.length > 0) && (
                <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-2">
                  {categoryFilter !== 'ignored' && githubDiff.length > 0 && (
                    <div className="flex flex-wrap items-center gap-3 text-xs">
                      <button
                        type="button"
                        onClick={() => handleSelectScope('all', () => true)}
                        className={`text-[#1D4ED8] dark:text-[#60A5FA] hover:underline whitespace-nowrap ${
                          selectionScope === 'all' ? 'font-semibold underline' : ''
                        }`}
                      >
                        Select All
                      </button>
                      <button
                        type="button"
                        onClick={() => handleSelectScope('pushable', (f) => f.canPush)}
                        className={`text-[#1D4ED8] dark:text-[#60A5FA] hover:underline whitespace-nowrap ${
                          selectionScope === 'pushable' ? 'font-semibold underline' : ''
                        }`}
                      >
                        Pushable
                      </button>
                      <button
                        type="button"
                        onClick={() => handleSelectScope('pullable', (f) => f.canPull)}
                        className={`text-[#1D4ED8] dark:text-[#60A5FA] hover:underline whitespace-nowrap ${
                          selectionScope === 'pullable' ? 'font-semibold underline' : ''
                        }`}
                      >
                        Pullable
                      </button>
                      {countsByCategory.superseded > 0 && (
                        <button
                          type="button"
                          onClick={() => {
                            setConfirmDirectPush(false);
                            setSelectionScope('superseded');
                            setSelectedPaths(
                              githubDiff
                                .filter((f) => f.category === 'superseded')
                                .map((f) => f.path)
                            );
                            setIncludeRemoteDeletions(true);
                          }}
                          className={`text-amber-700 dark:text-amber-400 hover:underline whitespace-nowrap ${
                            selectionScope === 'superseded' ? 'font-semibold underline' : ''
                          }`}
                        >
                          Superseded ({countsByCategory.superseded})
                        </button>
                      )}
                      <button
                        type="button"
                        onClick={() => {
                          setSelectionScope('none');
                          setFilterBySelectedScope(false);
                          setSelectedPaths([]);
                          setIncludeRemoteDeletions(false);
                        }}
                        className="text-[#57534E] dark:text-[#9CA3AF] hover:underline whitespace-nowrap"
                      >
                        Clear
                      </button>
                      <label
                        className={`inline-flex items-center gap-1.5 whitespace-nowrap select-none ${
                          isSelectAllSelected
                            ? 'text-[#A8A29E] dark:text-[#4B5563] cursor-not-allowed'
                            : 'text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2] cursor-pointer'
                        }`}
                        title={
                          isSelectAllSelected
                            ? 'Disabled while Select All is active'
                            : 'Filter list to the selected category'
                        }
                      >
                        <input
                          type="checkbox"
                          checked={filterBySelectedScope && !isSelectAllSelected}
                          disabled={isSelectAllSelected}
                          onChange={(e) => setFilterBySelectedScope(e.target.checked)}
                          className="rounded border-[#D6D3D1] dark:border-[#374151] disabled:opacity-50"
                        />
                        <span>Filter</span>
                      </label>
                    </div>
                  )}

                  <div className="relative w-full md:w-1/4 md:ml-auto shrink-0">
                    <Search className="w-3.5 h-3.5 text-[#57534E] dark:text-[#9CA3AF] absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
                    <input
                      type="text"
                      value={fileFilterInput}
                      onChange={(e) => setFileFilterInput(e.target.value)}
                      placeholder="Filter files..."
                      aria-label="Filter files by name or path"
                      className="w-full pl-9 pr-8 py-1.5 rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] text-[#1C1917] dark:text-[#ECEEF2] placeholder:text-[#78716C] dark:placeholder:text-[#6B7280] font-mono text-xs focus:outline-none focus:ring-2 focus:ring-[#1D4ED8]/40 focus:border-[#1D4ED8]"
                    />
                    {fileFilterInput && (
                      <button
                        type="button"
                        onClick={() => {
                          setFileFilterInput('');
                          setDebouncedFileFilter('');
                        }}
                        aria-label="Clear file filter"
                        className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2]"
                      >
                        <X className="w-3.5 h-3.5" />
                      </button>
                    )}
                  </div>
                </div>
              )}

              {categoryFilter === 'ignored' ? (
                <div className="space-y-4">
                  <div className="rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] p-3.5 space-y-3">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <div>
                        <div className="text-xs font-medium">
                          <code className="font-mono">.gitignore</code> Rules ({gitignoreRules.length})
                        </div>
                        <div className="text-[11px] text-[#57534E] dark:text-[#9CA3AF]">
                          Files matching <code className="font-mono">.gitignore</code> stay hidden from active push/pull diffs. Core secret and build cache rules remain protected.
                        </div>
                      </div>
                    </div>

                    <form onSubmit={handleAddPatternSubmit} className="flex flex-wrap items-center gap-2">
                      <input
                        type="text"
                        value={newIgnorePattern}
                        onChange={(e) => setNewIgnorePattern(e.target.value)}
                        placeholder="Add file path or pattern (e.g. metadata.json or *.tmp)"
                        className="flex-1 min-w-[220px] px-3 py-1.5 rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] text-[#1C1917] dark:text-[#ECEEF2] placeholder:text-[#78716C] dark:placeholder:text-[#6B7280] font-mono text-xs focus:outline-none focus:ring-2 focus:ring-[#1D4ED8]/40 focus:border-[#1D4ED8]"
                      />
                      <button
                        type="submit"
                        disabled={gitignoreBusy || !newIgnorePattern.trim()}
                        className="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium rounded-md bg-[#1D4ED8] text-white hover:bg-[#1E40AF] disabled:opacity-50 whitespace-nowrap"
                      >
                        <Plus className="w-3.5 h-3.5" />
                        <span>Add Rule</span>
                      </button>
                    </form>

                    <div className="max-h-44 overflow-y-auto rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] divide-y divide-[#E5E0D5] dark:divide-[#262B36]">
                      {gitignoreRules.map((rule) => (
                        <div
                          key={rule.pattern}
                          className="flex items-center justify-between gap-3 px-3 py-2 text-xs"
                        >
                          <div className="flex items-center gap-2.5 min-w-0">
                            <code className="font-mono truncate">{rule.pattern}</code>
                            {rule.matchedCount > 0 && (
                              <span className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF] shrink-0">
                                ({rule.matchedCount})
                              </span>
                            )}
                          </div>
                          {rule.locked ? (
                            <span className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF] shrink-0">
                              protected
                            </span>
                          ) : (
                            <button
                              type="button"
                              disabled={gitignoreBusy}
                              onClick={() => handleGitignoreMutation('remove', rule.pattern)}
                              className="inline-flex items-center gap-1 px-2 py-1 rounded border border-[#E5E0D5] dark:border-[#262B36] text-[11px] text-red-700 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 disabled:opacity-50 whitespace-nowrap shrink-0"
                            >
                              <Trash2 className="w-3 h-3" />
                              <span>Remove</span>
                            </button>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="space-y-2">
                    <div className="text-xs font-medium">
                      Gitignored Files ({visibleIgnoredFiles.length}
                      {debouncedFileFilter ? ` of ${ignoredFiles.length}` : ''})
                    </div>
                    {visibleIgnoredFiles.length === 0 ? (
                      <div className="rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] p-3.5 text-xs text-[#57534E] dark:text-[#9CA3AF]">
                        {debouncedFileFilter
                          ? 'No gitignored files match that filter.'
                          : (
                            <>
                              No tracked or workspace files outside build/secret caches are currently matched by{' '}
                              <code className="font-mono">.gitignore</code>.
                            </>
                          )}
                      </div>
                    ) : (
                      <div className="max-h-60 overflow-y-auto rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] divide-y divide-[#E5E0D5] dark:divide-[#262B36]">
                        {visibleIgnoredFiles.map((file) => {
                          const untrackChecked = selectedUntrackPaths.includes(file.path);
                          return (
                            <div
                              key={file.path}
                              className="flex flex-wrap items-center justify-between gap-2 px-3.5 py-2.5 text-xs"
                            >
                              <div className="flex items-center gap-2.5 min-w-0 flex-1">
                                {file.existsRemote && (
                                  <input
                                    type="checkbox"
                                    checked={untrackChecked}
                                    onChange={() => toggleUntrackSelection(file.path)}
                                    title="Select to untrack (remove) this ignored file from GitHub main on push"
                                    className="rounded border-[#D6D3D1] shrink-0"
                                  />
                                )}
                                <span className="font-mono truncate">{file.path}</span>
                              </div>
                              <div className="flex flex-wrap items-center gap-2.5 shrink-0">
                                <span className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF]">
                                  rule: {file.matchedRule}
                                </span>
                                {file.existsRemote && (
                                  <span className="text-[11px] font-mono text-amber-700 dark:text-amber-400">
                                    tracked on main
                                  </span>
                                )}
                                <span className="text-[11px] font-mono tabular-nums text-[#57534E] dark:text-[#9CA3AF]">
                                  {file.sizeBytes} B
                                </span>
                                {!file.ruleLocked && (
                                  <button
                                    type="button"
                                    disabled={gitignoreBusy}
                                    onClick={() => handleGitignoreMutation('unignore', file.path)}
                                    className="inline-flex items-center gap-1 px-2 py-1 rounded border border-[#E5E0D5] dark:border-[#262B36] text-[11px] text-[#1D4ED8] dark:text-[#60A5FA] hover:bg-[#1D4ED8]/10 disabled:opacity-50 whitespace-nowrap"
                                  >
                                    <span>Unignore</span>
                                  </button>
                                )}
                              </div>
                            </div>
                          );
                        })}
                      </div>
                    )}
                    {visibleIgnoredFiles.some((f) => f.existsRemote) && (
                      <div className="text-[11px] text-[#57534E] dark:text-[#9CA3AF]">
                        Check any <span className="font-mono">tracked on main</span> file above to remove its copy from GitHub on your next push while keeping it locally.
                      </div>
                    )}
                  </div>
                </div>
              ) : visibleFiles.length === 0 ? (
                <div className="rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] p-3.5 text-xs text-[#57534E] dark:text-[#9CA3AF]">
                  {githubDiff.length === 0
                    ? 'Everything non-ignored matches exonomyapp/exonovel@main.'
                    : debouncedFileFilter || filterBySelectedScope
                      ? 'No files match that filter.'
                      : 'No differences in this category.'}
                </div>
              ) : (
                <div className="max-h-72 overflow-y-auto rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] divide-y divide-[#E5E0D5] dark:divide-[#262B36]">
                  {visibleFiles.map((file) => {
                    const checked = selectedPaths.includes(file.path);
                    const isPreviewOpen = previewPath === file.path;
                    const canIgnore = file.path !== '.gitignore';
                    return (
                      <div
                        key={file.path}
                        className="flex items-center justify-between gap-3 px-3.5 py-2.5 text-xs hover:bg-[#F3F0EA]/60 dark:hover:bg-[#1E232D]/50"
                      >
                        <label className="flex items-center gap-2.5 min-w-0 flex-1 cursor-pointer">
                          <input
                            type="checkbox"
                            checked={checked}
                            onChange={() => togglePathSelection(file.path)}
                            className="rounded border-[#D6D3D1] shrink-0"
                          />
                          <span className="font-mono truncate">{file.path}</span>
                        </label>
                        <div className="flex items-center gap-2 sm:gap-3 shrink-0">
                          {file.remoteUpdatedSinceBase && (
                            <span
                              className="text-[11px] font-mono text-amber-700 dark:text-amber-400 hidden sm:inline"
                              title="GitHub main updated this file since its last synced baseline"
                            >
                              remote updated
                            </span>
                          )}
                          <span className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF] hidden sm:inline">
                            {categoryDisplayLabel(file.category)}
                          </span>
                          <span className={`text-[11px] font-mono ${statusColorClass(file.status)}`}>
                            {statusDisplayLabel(file.status)}
                          </span>
                          <span className="text-[11px] font-mono tabular-nums text-[#57534E] dark:text-[#9CA3AF] hidden sm:inline">
                            {file.sizeBytes} B
                          </span>
                          <button
                            type="button"
                            onClick={() => handlePreviewFile(file.path)}
                            className={`inline-flex items-center gap-1 px-2 py-1 rounded border text-[11px] whitespace-nowrap ${
                              isPreviewOpen
                                ? 'border-[#1D4ED8] text-[#1D4ED8] dark:text-[#60A5FA]'
                                : 'border-[#E5E0D5] dark:border-[#262B36] text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2]'
                            }`}
                          >
                            <Eye className="w-3 h-3" />
                            <span>{isPreviewOpen ? 'Hide' : 'Diff'}</span>
                          </button>
                          {canIgnore && (
                            <button
                              type="button"
                              disabled={gitignoreBusy}
                              onClick={() => handleGitignoreMutation('add', file.path)}
                              title="Add this file path to .gitignore"
                              className="inline-flex items-center gap-1 px-2 py-1 rounded border border-[#E5E0D5] dark:border-[#262B36] text-[11px] text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2] disabled:opacity-50 whitespace-nowrap"
                            >
                              <EyeOff className="w-3 h-3" />
                              <span className="hidden sm:inline">Ignore</span>
                            </button>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}

              {mergeQueue.length > 0 && (
                <div className="rounded-md border border-amber-300/80 dark:border-amber-900/60 bg-[#FAF8F4] dark:bg-[#111317] p-4 space-y-4">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div>
                      <div className="text-xs font-semibold text-amber-800 dark:text-amber-300">
                        Merge Review Required ({mergeQueue.length} file(s))
                      </div>
                      <div className="text-[11px] text-[#57534E] dark:text-[#9CA3AF]">
                        Both GitHub <code className="font-mono">main</code> and your local workspace edited these files. Apply or save a merged version below so neither side&apos;s changes are overwritten.
                      </div>
                    </div>
                    <button
                      type="button"
                      onClick={() => setMergeQueue([])}
                      className="text-xs text-[#57534E] dark:text-[#9CA3AF] hover:underline"
                    >
                      Dismiss All
                    </button>
                  </div>

                  {mergeQueue.map((conflict) => {
                    const isBusy = resolvingMergePath === conflict.path;
                    const reasonLabel =
                      conflict.reason === 'remote_ahead_clean'
                        ? 'Non-overlapping edits auto-merged (review before applying)'
                        : conflict.reason === 'overlap'
                          ? 'Overlapping line conflict'
                          : conflict.reason === 'binary'
                            ? 'Binary file conflict'
                            : 'Two-way comparison (no prior synced baseline)';

                    return (
                      <div
                        key={conflict.path}
                        className="rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] p-3.5 space-y-3 text-xs"
                      >
                        <div className="flex flex-wrap items-center justify-between gap-2">
                          <div className="font-mono font-medium truncate">
                            {conflict.path}
                          </div>
                          <span className="text-[11px] font-mono text-amber-700 dark:text-amber-400">
                            {reasonLabel}
                          </span>
                        </div>

                        {conflict.isBinary ? (
                          <div className="flex flex-wrap items-center gap-2">
                            <button
                              type="button"
                              disabled={githubPulling}
                              onClick={() => handlePullFromMain([conflict.path])}
                              className="px-3 py-1.5 rounded bg-[#1D4ED8] hover:bg-[#1E40AF] text-white text-xs disabled:opacity-50"
                            >
                              Take Remote Binary from main
                            </button>
                            <button
                              type="button"
                              onClick={() =>
                                setMergeQueue((prev) =>
                                  prev.filter((item) => item.path !== conflict.path)
                                )
                              }
                              className="px-3 py-1.5 rounded border border-[#E5E0D5] dark:border-[#262B36] text-xs"
                            >
                              Keep Local Binary
                            </button>
                          </div>
                        ) : (
                          <>
                            <div className="grid gap-3 md:grid-cols-2">
                              <div>
                                <div className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF] mb-1">
                                  GitHub main ({conflict.remoteSha.slice(0, 8)})
                                </div>
                                <pre className="max-h-40 overflow-auto rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] p-2 text-[11px] font-mono whitespace-pre-wrap break-all">
                                  {conflict.remoteContent ?? ''}
                                </pre>
                              </div>
                              <div>
                                <div className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF] mb-1">
                                  Local Container
                                </div>
                                <pre className="max-h-40 overflow-auto rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] p-2 text-[11px] font-mono whitespace-pre-wrap break-all">
                                  {conflict.localContent ?? ''}
                                </pre>
                              </div>
                            </div>

                            <div className="space-y-1.5">
                              <div className="flex flex-wrap items-center justify-between gap-2">
                                <span className="text-[11px] font-medium text-[#57534E] dark:text-[#9CA3AF]">
                                  Merged Output (editable)
                                </span>
                                <div className="flex flex-wrap items-center gap-2 text-[11px]">
                                  {conflict.suggestedContent !== null && (
                                    <button
                                      type="button"
                                      onClick={() =>
                                        setMergeDrafts((prev) => ({
                                          ...prev,
                                          [conflict.path]: conflict.suggestedContent || '',
                                        }))
                                      }
                                      className="text-[#1D4ED8] dark:text-[#60A5FA] hover:underline"
                                    >
                                      Use Auto-Merged
                                    </button>
                                  )}
                                  {conflict.localContent !== null && (
                                    <button
                                      type="button"
                                      onClick={() =>
                                        setMergeDrafts((prev) => ({
                                          ...prev,
                                          [conflict.path]: conflict.localContent || '',
                                        }))
                                      }
                                      className="text-[#1D4ED8] dark:text-[#60A5FA] hover:underline"
                                    >
                                      Use Local
                                    </button>
                                  )}
                                  {conflict.remoteContent !== null && (
                                    <button
                                      type="button"
                                      onClick={() =>
                                        setMergeDrafts((prev) => ({
                                          ...prev,
                                          [conflict.path]: conflict.remoteContent || '',
                                        }))
                                      }
                                      className="text-[#1D4ED8] dark:text-[#60A5FA] hover:underline"
                                    >
                                      Use Remote
                                    </button>
                                  )}
                                </div>
                              </div>
                              <textarea
                                rows={8}
                                value={mergeDrafts[conflict.path] ?? ''}
                                onChange={(e) =>
                                  setMergeDrafts((prev) => ({
                                    ...prev,
                                    [conflict.path]: e.target.value,
                                  }))
                                }
                                className="w-full p-2.5 rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] text-[#1C1917] dark:text-[#ECEEF2] font-mono text-[11px] focus:outline-none focus:ring-2 focus:ring-[#1D4ED8]/40 focus:border-[#1D4ED8]"
                              />
                              <div className="flex flex-wrap items-center gap-2">
                                <button
                                  type="button"
                                  disabled={isBusy}
                                  onClick={() => handleResolveMerge(conflict)}
                                  className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded bg-[#1D4ED8] hover:bg-[#1E40AF] text-white text-xs font-medium disabled:opacity-50"
                                >
                                  <span>
                                    {isBusy ? 'Saving Merge...' : 'Apply Merge Locally'}
                                  </span>
                                </button>
                                <button
                                  type="button"
                                  disabled={isBusy}
                                  onClick={() =>
                                    setMergeQueue((prev) =>
                                      prev.filter((item) => item.path !== conflict.path)
                                    )
                                  }
                                  className="px-3 py-1.5 rounded border border-[#E5E0D5] dark:border-[#262B36] text-xs"
                                >
                                  Cancel
                                </button>
                              </div>
                            </div>
                          </>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}

              {previewPath && (
                <div className="rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] p-4 space-y-3">
                  <div className="flex items-center justify-between gap-3">
                    <div className="font-mono text-xs font-medium truncate">{previewPath}</div>
                    <button
                      type="button"
                      onClick={() => {
                        setPreviewPath(null);
                        setPreviewData(null);
                      }}
                      className="inline-flex items-center gap-1 text-xs text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2]"
                    >
                      <X className="w-3.5 h-3.5" />
                      <span>Close</span>
                    </button>
                  </div>

                  {previewLoading ? (
                    <div className="text-xs text-[#57534E] dark:text-[#9CA3AF]">
                      Loading file comparison...
                    </div>
                  ) : previewData ? (
                    previewData.isBinary ? (
                      <div className="text-xs text-[#57534E] dark:text-[#9CA3AF] font-mono">
                        Binary file · Local: {previewData.existsLocal ? `${previewData.localSize} B` : 'missing'} · Remote main: {previewData.existsRemote ? `${previewData.remoteSize} B` : 'missing'}
                      </div>
                    ) : (
                      <div className="space-y-2">
                        <div className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF]">
                          {previewData.hasBase
                            ? previewData.remoteUpdatedSinceBase
                              ? `Synced baseline: ${previewData.baseSha?.slice(0, 8)} · GitHub main updated (${previewData.remoteSha?.slice(0, 8)})`
                              : `Synced baseline: ${previewData.baseSha?.slice(0, 8)} (remote unchanged since baseline)`
                            : 'No prior synced baseline recorded yet'}
                        </div>
                        <div className="grid gap-3 md:grid-cols-2">
                          <div>
                            <div className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF] mb-1">
                              Remote main ({previewData.existsRemote ? `${previewData.remoteSize} B` : 'not on main'})
                            </div>
                            <pre className="max-h-60 overflow-auto rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] p-2.5 text-[11px] font-mono whitespace-pre-wrap break-all">
                              {previewData.remoteContent ?? '(absent on main)'}
                            </pre>
                          </div>
                          <div>
                            <div className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF] mb-1">
                              Local Container ({previewData.existsLocal ? `${previewData.localSize} B` : 'not local'})
                            </div>
                            <pre className="max-h-60 overflow-auto rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] p-2.5 text-[11px] font-mono whitespace-pre-wrap break-all">
                              {previewData.localContent ?? '(absent locally)'}
                            </pre>
                          </div>
                        </div>
                      </div>
                    )
                  ) : null}
                </div>
              )}
            </div>

            {(githubDiff.length > 0 || selectedUntrackPaths.length > 0) && (
              <div className="space-y-4 pt-3 border-t border-[#E5E0D5] dark:border-[#262B36]">
                {githubDiff.length > 0 && (
                  <div className="flex flex-wrap items-center justify-between gap-3 rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] p-3.5 text-xs">
                    <div>
                      <div className="font-medium">
                        Sync Down: <code className="font-mono">exonomyapp/exonovel@main</code> → Local Container
                      </div>
                      <div className="mt-0.5 text-[#57534E] dark:text-[#9CA3AF]">
                        Pulls <span className="font-mono">on main only</span> files and 3-way merges <span className="font-mono">modified</span> files against saved baselines so local edits stay preserved.
                        {selectedPullModifiedCount > 0 && (
                          <span className="text-[#1D4ED8] dark:text-[#60A5FA]">
                            {' '}({selectedPullModifiedCount} modified file(s) will merge or prompt on conflict)
                          </span>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <button
                        type="button"
                        disabled={githubPulling || selectedPullFiles.length === 0}
                        onClick={() => handlePullFromMain()}
                        className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium rounded-md border border-[#1D4ED8] text-[#1D4ED8] dark:text-[#60A5FA] hover:bg-[#1D4ED8]/10 transition-colors min-h-[40px] whitespace-nowrap disabled:opacity-50"
                      >
                        <Download className="w-3.5 h-3.5" />
                        <span>
                          {githubPulling
                            ? 'Merging & Pulling...'
                            : `Pull & Merge from main (${selectedPullFiles.length})`}
                        </span>
                      </button>
                    </div>
                  </div>
                )}

                {selectedRemoteDeleteCandidates.length > 0 && (
                  <label className="flex items-start gap-2.5 rounded-md border border-amber-300/80 dark:border-amber-900/60 bg-amber-50/70 dark:bg-amber-950/20 p-3 text-xs cursor-pointer">
                    <input
                      type="checkbox"
                      checked={includeRemoteDeletions}
                      onChange={(e) => setIncludeRemoteDeletions(e.target.checked)}
                      className="mt-0.5 rounded border-[#D6D3D1]"
                    />
                    <span>
                      <span className="font-medium inline-flex items-center gap-1">
                        <Trash2 className="w-3.5 h-3.5 text-amber-700 dark:text-amber-400" />
                        Remove {selectedRemoteDeleteCandidates.length} checked &ldquo;on main only&rdquo; file(s) from GitHub on push
                      </span>
                      <span className="block mt-0.5 text-[#57534E] dark:text-[#9CA3AF]">
                        Optional cleanup for superseded files. Leave unchecked to keep remote-only files untouched on <code className="font-mono">main</code>.
                      </span>
                    </span>
                  </label>
                )}

                {selectedUntrackPaths.length > 0 && (
                  <div className="rounded-md border border-amber-300/80 dark:border-amber-900/60 bg-amber-50/70 dark:bg-amber-950/20 p-3 text-xs">
                    {selectedUntrackPaths.length} gitignored file(s) selected to untrack from <code className="font-mono">main</code> on your next push (local copies stay intact).
                  </div>
                )}

                <div className="grid gap-3 sm:grid-cols-2 text-xs">
                  <div>
                    <label className="block text-[#57534E] dark:text-[#9CA3AF] mb-1">
                      Commit Message
                    </label>
                    <input
                      type="text"
                      value={commitMessage}
                      onChange={(e) => setCommitMessage(e.target.value)}
                      placeholder={suggestedCommit || 'chore: update exonomyapp/exonovel'}
                      className="w-full px-3 py-2 rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] font-mono text-xs"
                    />
                  </div>
                  <div>
                    <label className="block text-[#57534E] dark:text-[#9CA3AF] mb-1">
                      PR Branch Name
                    </label>
                    <div className="flex items-center gap-1.5">
                      <GitBranch className="w-3.5 h-3.5 text-[#57534E] shrink-0" />
                      <input
                        type="text"
                        value={branchName}
                        onChange={(e) => setBranchName(e.target.value)}
                        className="w-full px-3 py-2 rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] font-mono text-xs"
                      />
                    </div>
                  </div>
                  <div className="sm:col-span-2">
                    <label className="block text-[#57534E] dark:text-[#9CA3AF] mb-1">
                      Pull Request Title
                    </label>
                    <input
                      type="text"
                      value={prTitle}
                      onChange={(e) => setPrTitle(e.target.value)}
                      placeholder={suggestedCommit || 'Summary of changes for exonomyapp/exonovel'}
                      className="w-full px-3 py-2 rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] text-xs"
                    />
                  </div>

                  <div className="sm:col-span-2 flex flex-wrap items-center gap-2 pt-1">
                    <button
                      type="button"
                      disabled={githubSyncing || effectivePushCount === 0}
                      onClick={() => handleGitHubSync('pr')}
                      className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium rounded-md bg-[#1D4ED8] text-white hover:bg-[#1E40AF] transition-colors min-h-[40px] whitespace-nowrap shrink-0 disabled:opacity-50"
                    >
                      <GitPullRequest className="w-3.5 h-3.5" />
                      <span>
                        {githubSyncing
                          ? 'Syncing...'
                          : `Create Pull Request (${selectedPushFiles.length} push${
                              allDeletePaths.length > 0
                                ? `, ${allDeletePaths.length} remove`
                                : ''
                            })`}
                      </span>
                    </button>

                    {!confirmDirectPush ? (
                      <button
                        type="button"
                        disabled={githubSyncing || effectivePushCount === 0}
                        onClick={() => setConfirmDirectPush(true)}
                        className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] hover:bg-[#EAE5DB] dark:hover:bg-[#1E232D] min-h-[40px] whitespace-nowrap shrink-0 disabled:opacity-50"
                      >
                        <Upload className="w-3.5 h-3.5" />
                        <span>Push Directly to main</span>
                      </button>
                    ) : (
                      <div className="inline-flex items-center gap-2">
                        <button
                          type="button"
                          disabled={githubSyncing}
                          onClick={() => handleGitHubSync('direct')}
                          className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium rounded-md bg-amber-700 hover:bg-amber-800 text-white min-h-[40px] whitespace-nowrap shrink-0 disabled:opacity-50"
                        >
                          <Upload className="w-3.5 h-3.5" />
                          <span>Confirm Push to main</span>
                        </button>
                        <button
                          type="button"
                          disabled={githubSyncing}
                          onClick={() => setConfirmDirectPush(false)}
                          className="px-3.5 py-2 text-xs rounded-md border border-[#E5E0D5] dark:border-[#262B36] min-h-[40px] whitespace-nowrap shrink-0"
                        >
                          Cancel
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}

            {githubStatus.openPulls.length > 0 && (
              <div className="pt-4 border-t border-[#E5E0D5] dark:border-[#262B36]">
                <div className="text-xs font-medium mb-2">
                  Open Pull Requests on <code className="font-mono">exonomyapp/exonovel</code>
                </div>
                <div className="rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] divide-y divide-[#E5E0D5] dark:divide-[#262B36]">
                  {githubStatus.openPulls.map((pr) => (
                    <div
                      key={pr.number}
                      className="flex flex-wrap items-center justify-between gap-3 px-3.5 py-2.5 text-xs"
                    >
                      <div className="min-w-0">
                        <div className="font-medium">
                          #{pr.number} {pr.title}
                        </div>
                        <div className="text-[#57534E] dark:text-[#9CA3AF] font-mono text-[11px]">
                          {pr.headRef} → {pr.baseRef} · by @{pr.userLogin}
                        </div>
                      </div>
                      <div className="flex items-center gap-2 shrink-0">
                        <a
                          href={pr.htmlUrl}
                          target="_blank"
                          rel="noreferrer"
                          className="inline-flex items-center gap-1 px-3 py-1.5 rounded border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] whitespace-nowrap"
                        >
                          <span>GitHub</span>
                          <ExternalLink className="w-3 h-3" />
                        </a>
                        <button
                          type="button"
                          disabled={mergingPr === pr.number}
                          onClick={() => handleMergePullRequest(pr.number)}
                          className="inline-flex items-center gap-1 px-3 py-1.5 rounded bg-emerald-700 hover:bg-emerald-800 text-white whitespace-nowrap disabled:opacity-50"
                        >
                          <GitMerge className="w-3 h-3" />
                          <span>
                            {mergingPr === pr.number ? 'Merging...' : 'Merge PR'}
                          </span>
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
