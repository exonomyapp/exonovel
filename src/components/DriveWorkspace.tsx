import { type ChangeEvent, useCallback, useEffect, useState } from 'react';
import type { User } from 'firebase/auth';
import {
  Activity,
  Archive,
  ArrowLeft,
  ChevronRight,
  ExternalLink,
  Eye,
  FileText,
  FileUp,
  Folder,
  FolderPlus,
  HardDrive,
  LogOut,
  MessageSquareText,
  Pencil,
  RefreshCw,
  RotateCcw,
  Search,
  Trash2,
  Upload,
  X,
} from 'lucide-react';
import type { DocNode, DocPayload } from '../types/docs';
import {
  backupDocsArchiveToDrive,
  createDriveFolder,
  deleteDriveFilePermanently,
  DriveAuthExpiredError,
  ensureDriveFolder,
  listDriveActivity,
  listDriveFiles,
  readDriveFileText,
  renameDriveFile,
  setDriveFileTrashed,
  upsertMarkdownFileInDrive,
  type DriveActivityEntry,
  type DriveFile,
} from '../lib/driveApi';
import {
  getDriveAccessToken,
  googleDriveSignIn,
  initDriveAuth,
  logoutDrive,
} from '../lib/driveAuth';
import { GoogleSignInButton } from './GoogleSignInButton';

interface DriveWorkspaceProps {
  activeDoc: DocPayload | null;
  allDocs: DocNode[];
  getToken: () => Promise<string | null>;
  googleSsoLinked?: boolean;
  googleSsoEmail?: string | null;
  initialAuthError?: string | null;
  onDriveAuthChange?: (connected: boolean) => void;
  onOpenSettings?: () => void;
  onBackToDocs: () => void;
}

interface FolderCrumb {
  id: string;
  name: string;
}

type PendingMutation =
  | {
      kind: 'upload_active_doc';
      fileName: string;
      content: string;
      folderName: string;
      parentId: string;
    }
  | {
      kind: 'backup_all_docs';
      folderName: string;
      parentId: string;
      parentFolderLabel: string;
      docPaths: string[];
    }
  | {
      kind: 'upload_transcript';
      fileName: string;
      content: string;
      useTranscriptSubfolder: boolean;
      parentId: string;
      parentFolderLabel: string;
    }
  | {
      kind: 'create_folder';
      folderName: string;
      parentId: string;
    }
  | {
      kind: 'rename';
      file: DriveFile;
      newName: string;
    }
  | {
      kind: 'trash';
      file: DriveFile;
    }
  | {
      kind: 'restore';
      file: DriveFile;
    }
  | {
      kind: 'delete_permanently';
      file: DriveFile;
    };

function formatFileSize(raw?: string): string {
  if (!raw) return '—';
  const bytes = Number(raw);
  if (!Number.isFinite(bytes)) return '—';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(raw?: string): string {
  if (!raw) return '—';
  const parsed = new Date(raw);
  if (Number.isNaN(parsed.getTime())) return raw;
  return parsed.toLocaleString();
}

function isPreviewableTextFile(file: DriveFile): boolean {
  return (
    file.mimeType.startsWith('text/') ||
    file.mimeType === 'application/json' ||
    file.mimeType === 'application/vnd.google-apps.document' ||
    file.mimeType === 'application/vnd.google-apps.spreadsheet' ||
    file.mimeType === 'application/vnd.google-apps.presentation' ||
    /\.(md|markdown|txt|json|csv|yml|yaml|toml|rs|ts|tsx)$/i.test(file.name)
  );
}

function normalizeTranscriptFileName(raw: string): string {
  const trimmed = raw.trim() || `exonovel-chat-${new Date().toISOString().slice(0, 10)}.md`;
  if (/\.(md|markdown|txt)$/i.test(trimmed)) {
    return trimmed.replace(/\.(txt|markdown)$/i, '.md');
  }
  return `${trimmed.replace(/\.json$/i, '')}.md`;
}

function formatTranscriptMarkdown(rawText: string, fileName: string): string {
  const trimmed = rawText.trim();
  if (!trimmed) return '';

  const headingTitle = fileName.replace(/\.(md|markdown|txt)$/i, '');
  const savedStamp = new Date().toLocaleString();

  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    try {
      const parsed = JSON.parse(trimmed) as {
        chunkedPrompt?: { chunks?: Array<{ role?: string; text?: string; isThought?: boolean }> };
        messages?: Array<{ role?: string; text?: string; content?: string }>;
      };
      const chunks = parsed?.chunkedPrompt?.chunks;
      if (Array.isArray(chunks) && chunks.length > 0) {
        const lines: string[] = [
          `# ${headingTitle}`,
          '',
          `- Saved: ${savedStamp}`,
          '- Source: Google AI Studio Chat Export',
          '',
          '---',
          '',
        ];
        for (const chunk of chunks) {
          if (chunk.isThought || !chunk.text?.trim()) continue;
          const label = chunk.role === 'model' ? 'Assistant' : chunk.role === 'user' ? 'Owner' : chunk.role || 'Message';
          lines.push(`## ${label}`, '', chunk.text.trim(), '', '---', '');
        }
        return lines.join('\n');
      }

      const messages = Array.isArray(parsed) ? parsed : parsed?.messages;
      if (Array.isArray(messages) && messages.length > 0) {
        const lines: string[] = [
          `# ${headingTitle}`,
          '',
          `- Saved: ${savedStamp}`,
          '- Source: Chat Transcript Export',
          '',
          '---',
          '',
        ];
        for (const msg of messages as Array<{ role?: string; text?: string; content?: string }>) {
          const body = msg.text || msg.content;
          if (!body || typeof body !== 'string') continue;
          const label = msg.role === 'model' || msg.role === 'assistant' ? 'Assistant' : msg.role === 'user' ? 'Owner' : msg.role || 'Message';
          lines.push(`## ${label}`, '', body.trim(), '', '---', '');
        }
        if (lines.length > 7) {
          return lines.join('\n');
        }
      }
    } catch {
      // Fall back to standard text formatting if JSON parsing fails
    }
  }

  if (trimmed.startsWith('# ')) {
    return trimmed;
  }

  return [
    `# ${headingTitle}`,
    '',
    `- Saved: ${savedStamp}`,
    '- Source: Google AI Studio Build Session Transcript',
    '',
    '---',
    '',
    trimmed,
    '',
  ].join('\n');
}

export function DriveWorkspace({
  activeDoc,
  allDocs,
  getToken,
  googleSsoLinked = false,
  googleSsoEmail = null,
  initialAuthError = null,
  onDriveAuthChange,
  onOpenSettings,
  onBackToDocs,
}: DriveWorkspaceProps) {
  const [driveUser, setDriveUser] = useState<User | null>(null);
  const [needsAuth, setNeedsAuth] = useState<boolean>(() => !getDriveAccessToken());
  const [signingIn, setSigningIn] = useState(false);

  const [tab, setTab] = useState<'files' | 'transcript' | 'trash' | 'activity'>('files');
  const [folderStack, setFolderStack] = useState<FolderCrumb[]>([
    { id: 'root', name: 'My Drive' },
  ]);
  const [searchInput, setSearchInput] = useState('');
  const [activeSearch, setActiveSearch] = useState('');

  const [files, setFiles] = useState<DriveFile[]>([]);
  const [activities, setActivities] = useState<DriveActivityEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(initialAuthError);
  const [statusNotice, setStatusNotice] = useState<string | null>(null);
  const [backupProgress, setBackupProgress] = useState<string | null>(null);

  const [previewFile, setPreviewFile] = useState<{ file: DriveFile; text: string } | null>(null);
  const [loadingPreviewId, setLoadingPreviewId] = useState<string | null>(null);

  // Chat transcript saver state (Option 2)
  const [transcriptFileName, setTranscriptFileName] = useState<string>(
    () => `exonovel-chat-${new Date().toISOString().slice(0, 10)}.md`
  );
  const [transcriptText, setTranscriptText] = useState('');
  const [useTranscriptSubfolder, setUseTranscriptSubfolder] = useState(true);

  // Input prompts before confirmation modal
  const [newFolderDraft, setNewFolderDraft] = useState<string | null>(null);
  const [renameDraft, setRenameDraft] = useState<{ file: DriveFile; value: string } | null>(null);

  // Mandatory confirmation dialog for every mutating or destructive Drive API operation
  const [pendingMutation, setPendingMutation] = useState<PendingMutation | null>(null);
  const [executingMutation, setExecutingMutation] = useState(false);

  const currentFolder = folderStack[folderStack.length - 1];

  useEffect(() => {
    if (initialAuthError) {
      setError(initialAuthError);
    }
  }, [initialAuthError]);

  useEffect(() => {
    const unsubscribe = initDriveAuth(
      (user) => {
        setDriveUser(user);
        setNeedsAuth(false);
        onDriveAuthChange?.(true);
      },
      () => {
        setDriveUser(null);
        setNeedsAuth(true);
        onDriveAuthChange?.(false);
      }
    );
    return unsubscribe;
  }, [onDriveAuthChange]);

  const loadDriveData = useCallback(async () => {
    const token = getDriveAccessToken();
    if (!token) {
      setNeedsAuth(true);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      if (tab === 'transcript') {
        setLoading(false);
        return;
      }
      if (tab === 'activity') {
        const items = await listDriveActivity(token, 25);
        setActivities(items);
      } else {
        const list = await listDriveFiles(token, {
          folderId: currentFolder.id,
          searchQuery: activeSearch,
          trashedOnly: tab === 'trash',
        });
        setFiles(list);
      }
    } catch (err) {
      if (err instanceof DriveAuthExpiredError) {
        setNeedsAuth(true);
        setDriveUser(null);
      }
      setError(err instanceof Error ? err.message : 'Unable to reach Google Drive.');
    } finally {
      setLoading(false);
    }
  }, [activeSearch, currentFolder.id, tab]);

  useEffect(() => {
    if (!needsAuth && tab !== 'transcript') {
      loadDriveData();
    }
  }, [needsAuth, tab, loadDriveData]);

  const handleGoogleSignIn = async () => {
    setSigningIn(true);
    setError(null);
    try {
      const { user } = await googleDriveSignIn({
        loginHint: googleSsoLinked && googleSsoEmail ? googleSsoEmail : undefined,
      });
      setDriveUser(user);
      setNeedsAuth(false);
      onDriveAuthChange?.(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Google sign-in failed.');
    } finally {
      setSigningIn(false);
    }
  };

  const handleDisconnect = async () => {
    await logoutDrive();
    setDriveUser(null);
    setNeedsAuth(true);
    setFiles([]);
    setActivities([]);
    setPreviewFile(null);
    onDriveAuthChange?.(false);
  };

  const handlePreview = async (file: DriveFile) => {
    const token = getDriveAccessToken();
    if (!token) {
      setNeedsAuth(true);
      return;
    }
    setLoadingPreviewId(file.id);
    setError(null);
    try {
      const text = await readDriveFileText(token, file);
      setPreviewFile({ file, text });
    } catch (err) {
      if (err instanceof DriveAuthExpiredError) {
        setNeedsAuth(true);
      }
      setError(err instanceof Error ? err.message : `Could not open ${file.name}.`);
    } finally {
      setLoadingPreviewId(null);
    }
  };

  const handleTranscriptFileUpload = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    try {
      const content = await file.text();
      setTranscriptText(content);
      setTranscriptFileName(normalizeTranscriptFileName(file.name));
      setError(null);
    } catch {
      setError(`Unable to read "${file.name}". Please try another text or Markdown file.`);
    } finally {
      event.target.value = '';
    }
  };

  const confirmAndExecuteMutation = async () => {
    if (!pendingMutation) return;
    const token = getDriveAccessToken();
    if (!token) {
      setPendingMutation(null);
      setNeedsAuth(true);
      return;
    }

    setExecutingMutation(true);
    setError(null);
    setStatusNotice(null);
    setBackupProgress(null);
    try {
      switch (pendingMutation.kind) {
        case 'upload_active_doc': {
          const result = await upsertMarkdownFileInDrive(token, {
            name: pendingMutation.fileName,
            content: pendingMutation.content,
            parentId: pendingMutation.parentId,
          });
          setStatusNotice(
            `${result.action === 'updated' ? 'Updated' : 'Saved'} "${result.file.name}" in ${pendingMutation.folderName} on Google Drive.`
          );
          break;
        }
        case 'backup_all_docs': {
          const authToken = await getToken();
          const fetchedDocs: Array<{ path: string; content: string }> = [];
          for (let i = 0; i < pendingMutation.docPaths.length; i += 1) {
            const docPath = pendingMutation.docPaths[i];
            setBackupProgress(
              `Preparing document ${i + 1} of ${pendingMutation.docPaths.length}: ${docPath}`
            );
            const res = await fetch(`/api/doc?path=${encodeURIComponent(docPath)}`, {
              cache: 'no-store',
              headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
            });
            if (!res.ok) {
              throw new Error(`Unable to read "${docPath}" for backup.`);
            }
            const payload = (await res.json()) as DocPayload;
            fetchedDocs.push({ path: docPath, content: payload.content });
          }

          const summary = await backupDocsArchiveToDrive(token, {
            rootParentId: pendingMutation.parentId,
            rootFolderName: pendingMutation.folderName,
            docs: fetchedDocs,
            onProgress: (done, total, currentPath) => {
              setBackupProgress(`Uploading ${done} of ${total} to Google Drive: ${currentPath}`);
            },
          });

          setStatusNotice(
            `Backed up all ${summary.total} ExoNovel documents into "${pendingMutation.folderName}" (${summary.createdCount} new, ${summary.updatedCount} updated).`
          );
          break;
        }
        case 'upload_transcript': {
          let targetParentId = pendingMutation.parentId;
          let targetLabel = pendingMutation.parentFolderLabel;
          if (pendingMutation.useTranscriptSubfolder) {
            const ensured = await ensureDriveFolder(
              token,
              'ExoNovel Chat Transcripts',
              pendingMutation.parentId
            );
            targetParentId = ensured.folder.id;
            targetLabel = `${pendingMutation.parentFolderLabel} / ExoNovel Chat Transcripts`;
          }
          const result = await upsertMarkdownFileInDrive(token, {
            name: pendingMutation.fileName,
            content: pendingMutation.content,
            mimeType: 'text/markdown',
            parentId: targetParentId,
          });
          setTranscriptText('');
          setStatusNotice(
            `${result.action === 'updated' ? 'Updated' : 'Saved'} chat transcript "${result.file.name}" inside "${targetLabel}".`
          );
          break;
        }
        case 'create_folder': {
          const created = await createDriveFolder(
            token,
            pendingMutation.folderName,
            pendingMutation.parentId
          );
          setStatusNotice(`Created folder "${created.name}" in Google Drive.`);
          break;
        }
        case 'rename': {
          const renamed = await renameDriveFile(
            token,
            pendingMutation.file.id,
            pendingMutation.newName
          );
          setStatusNotice(`Renamed file to "${renamed.name}".`);
          break;
        }
        case 'trash': {
          await setDriveFileTrashed(token, pendingMutation.file.id, true);
          setStatusNotice(`Moved "${pendingMutation.file.name}" to Google Drive Trash.`);
          break;
        }
        case 'restore': {
          await setDriveFileTrashed(token, pendingMutation.file.id, false);
          setStatusNotice(`Restored "${pendingMutation.file.name}" from Trash.`);
          break;
        }
        case 'delete_permanently': {
          await deleteDriveFilePermanently(token, pendingMutation.file.id);
          if (previewFile?.file.id === pendingMutation.file.id) {
            setPreviewFile(null);
          }
          setStatusNotice(
            `Permanently deleted "${pendingMutation.file.name}" from Google Drive.`
          );
          break;
        }
      }
      setPendingMutation(null);
      if (tab !== 'transcript') {
        await loadDriveData();
      }
    } catch (err) {
      if (err instanceof DriveAuthExpiredError) {
        setNeedsAuth(true);
      }
      setError(err instanceof Error ? err.message : 'Google Drive action failed.');
      setPendingMutation(null);
    } finally {
      setExecutingMutation(false);
      setBackupProgress(null);
    }
  };

  const getMutationPromptCopy = (m: PendingMutation): {
    title: string;
    detail: string;
    confirmLabel: string;
    destructive: boolean;
  } => {
    switch (m.kind) {
      case 'upload_active_doc':
        return {
          title: 'Save document to Google Drive?',
          detail: `This will upload or update "${m.fileName}" inside folder "${m.folderName}" on your Google Drive.`,
          confirmLabel: 'Save to Drive',
          destructive: false,
        };
      case 'backup_all_docs':
        return {
          title: 'Back up all ExoNovel docs to Google Drive?',
          detail: `This will create or update the folder "${m.folderName}" inside "${m.parentFolderLabel}" and upload all ${m.docPaths.length} documents (session.md, scope.md, AGENTS.md, and all ADRs), keeping subfolder structure intact.`,
          confirmLabel: `Back Up ${m.docPaths.length} Documents`,
          destructive: false,
        };
      case 'upload_transcript':
        return {
          title: 'Save chat transcript to Google Drive?',
          detail: m.useTranscriptSubfolder
            ? `This will save "${m.fileName}" inside "${m.parentFolderLabel} / ExoNovel Chat Transcripts" on your Google Drive.`
            : `This will save "${m.fileName}" directly inside "${m.parentFolderLabel}" on your Google Drive.`,
          confirmLabel: 'Save Transcript',
          destructive: false,
        };
      case 'create_folder':
        return {
          title: 'Create folder in Google Drive?',
          detail: `This will create a new folder named "${m.folderName}" inside "${currentFolder.name}".`,
          confirmLabel: 'Create Folder',
          destructive: false,
        };
      case 'rename':
        return {
          title: 'Rename file in Google Drive?',
          detail: `Rename "${m.file.name}" to "${m.newName}"?`,
          confirmLabel: 'Confirm Rename',
          destructive: false,
        };
      case 'trash':
        return {
          title: 'Move item to Google Drive Trash?',
          detail: `Move "${m.file.name}" to Google Drive Trash? You can restore it later from the Trash tab.`,
          confirmLabel: 'Move to Trash',
          destructive: true,
        };
      case 'restore':
        return {
          title: 'Restore item from Trash?',
          detail: `Restore "${m.file.name}" back into your Google Drive?`,
          confirmLabel: 'Restore File',
          destructive: false,
        };
      case 'delete_permanently':
        return {
          title: 'Permanently delete from Google Drive?',
          detail: `Are you sure you want to permanently delete "${m.file.name}"? This action cannot be undone.`,
          confirmLabel: 'Delete Permanently',
          destructive: true,
        };
    }
  };

  return (
    <div className="max-w-5xl mx-auto space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-4 border-b border-[#E5E0D5] dark:border-[#262B36] pb-5">
        <div>
          <div className="flex items-center gap-2.5">
            <HardDrive className="w-5 h-5 text-[#1D4ED8] dark:text-[#60A5FA]" />
            <h1 className="font-display text-2xl font-semibold tracking-tight">
              Google Drive Workspace
            </h1>
          </div>
          <p className="mt-1 text-sm text-[#57534E] dark:text-[#9CA3AF]">
            Browse your Drive files, inspect recent Drive activity, or export ExoNovel docs safely
            with explicit confirmation.
          </p>
        </div>

        <button
          type="button"
          onClick={onBackToDocs}
          className="inline-flex items-center gap-2 px-3.5 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors min-h-[40px]"
        >
          <ArrowLeft className="w-3.5 h-3.5" />
          Back to Docs
        </button>
      </div>

      {error && (
        <div className="rounded-md border border-red-300 dark:border-red-900/60 bg-red-50 dark:bg-red-950/30 px-4 py-3 text-sm text-red-800 dark:text-red-200 flex items-center justify-between gap-3">
          <span>{error}</span>
          <button type="button" onClick={() => setError(null)} aria-label="Dismiss error">
            <X className="w-4 h-4" />
          </button>
        </div>
      )}

      {statusNotice && (
        <div className="rounded-md border border-emerald-300 dark:border-emerald-900/60 bg-emerald-50 dark:bg-emerald-950/30 px-4 py-3 text-sm text-emerald-800 dark:text-emerald-200 flex items-center justify-between gap-3">
          <span>{statusNotice}</span>
          <button
            type="button"
            onClick={() => setStatusNotice(null)}
            aria-label="Dismiss notice"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      )}

      {needsAuth ? (
        <div className="rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/60 dark:bg-[#171A21] p-8 space-y-5">
          <div className="space-y-2 max-w-xl">
            <h2 className="font-display text-lg font-semibold">
              Connect your Google account for Drive access
            </h2>
            {googleSsoLinked && googleSsoEmail && (
              <p className="text-xs font-medium text-[#1D4ED8] dark:text-[#60A5FA]">
                Detected ExoNovel Google login ({googleSsoEmail}) — pre-selected for quick Drive
                authorization.
              </p>
            )}
            <p className="text-sm text-[#57534E] dark:text-[#9CA3AF]">
              Sign in with Google to grant Drive permission, browse files, back up ExoNovel docs,
              and save transcripts. You can manage or revoke this connection anytime under{' '}
              {onOpenSettings ? (
                <button
                  type="button"
                  onClick={onOpenSettings}
                  className="underline font-medium text-[#1D4ED8] dark:text-[#60A5FA]"
                >
                  Settings
                </button>
              ) : (
                'Settings'
              )}
              .
            </p>
          </div>
          <GoogleSignInButton onClick={handleGoogleSignIn} disabled={signingIn} />
        </div>
      ) : (
        <div className="space-y-5">
          <div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/60 dark:bg-[#171A21] px-4 py-3">
            <div className="text-xs sm:text-sm text-[#44403C] dark:text-[#D1D5DB]">
              Connected to Google Drive as{' '}
              <span className="font-semibold text-[#1C1917] dark:text-[#ECEEF2]">
                {driveUser?.email || driveUser?.displayName || 'Google User'}
              </span>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              {allDocs.length > 0 && (
                <button
                  type="button"
                  onClick={() =>
                    setPendingMutation({
                      kind: 'backup_all_docs',
                      folderName: 'ExoNovel Docs',
                      parentId: currentFolder.id,
                      parentFolderLabel: currentFolder.name,
                      docPaths: allDocs.map((d) => d.path),
                    })
                  }
                  className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md bg-[#1D4ED8] text-white hover:bg-[#1E40AF] transition-colors min-h-[38px]"
                >
                  <Archive className="w-3.5 h-3.5" />
                  Back Up All Docs ({allDocs.length})
                </button>
              )}
              {activeDoc && (
                <button
                  type="button"
                  onClick={() =>
                    setPendingMutation({
                      kind: 'upload_active_doc',
                      fileName: activeDoc.name.endsWith('.md')
                        ? activeDoc.name
                        : `${activeDoc.name}.md`,
                      content: activeDoc.content,
                      folderName: currentFolder.name,
                      parentId: currentFolder.id,
                    })
                  }
                  className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md border border-[#1D4ED8]/40 dark:border-[#60A5FA]/40 text-[#1D4ED8] dark:text-[#60A5FA] bg-[#EFF6FF] dark:bg-[#172554]/30 hover:bg-[#DBEAFE] dark:hover:bg-[#172554]/60 transition-colors min-h-[38px]"
                >
                  <Upload className="w-3.5 h-3.5" />
                  Save "{activeDoc.name}"
                </button>
              )}
              <button
                type="button"
                onClick={() => setNewFolderDraft('')}
                className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] hover:bg-[#EAE5DB] dark:hover:bg-[#1E232D] transition-colors min-h-[38px]"
              >
                <FolderPlus className="w-3.5 h-3.5" />
                New Folder
              </button>
              <button
                type="button"
                onClick={handleDisconnect}
                className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] hover:bg-[#EAE5DB] dark:hover:bg-[#1E232D] transition-colors min-h-[38px]"
              >
                <LogOut className="w-3.5 h-3.5" />
                Disconnect
              </button>
            </div>
          </div>

          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="inline-flex flex-wrap rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/60 dark:bg-[#171A21] p-1 gap-1">
              <button
                type="button"
                onClick={() => setTab('files')}
                className={`px-3 py-1.5 text-xs font-medium rounded transition-colors ${
                  tab === 'files'
                    ? 'bg-white dark:bg-[#1E2430] text-[#1D4ED8] dark:text-[#60A5FA]'
                    : 'text-[#57534E] dark:text-[#9CA3AF]'
                }`}
              >
                Files
              </button>
              <button
                type="button"
                onClick={() => setTab('transcript')}
                className={`inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded transition-colors ${
                  tab === 'transcript'
                    ? 'bg-white dark:bg-[#1E2430] text-[#1D4ED8] dark:text-[#60A5FA]'
                    : 'text-[#57534E] dark:text-[#9CA3AF]'
                }`}
              >
                <MessageSquareText className="w-3.5 h-3.5" />
                Save Chat Transcript
              </button>
              <button
                type="button"
                onClick={() => setTab('activity')}
                className={`inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded transition-colors ${
                  tab === 'activity'
                    ? 'bg-white dark:bg-[#1E2430] text-[#1D4ED8] dark:text-[#60A5FA]'
                    : 'text-[#57534E] dark:text-[#9CA3AF]'
                }`}
              >
                <Activity className="w-3.5 h-3.5" />
                Recent Activity
              </button>
              <button
                type="button"
                onClick={() => setTab('trash')}
                className={`inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded transition-colors ${
                  tab === 'trash'
                    ? 'bg-white dark:bg-[#1E2430] text-[#1D4ED8] dark:text-[#60A5FA]'
                    : 'text-[#57534E] dark:text-[#9CA3AF]'
                }`}
              >
                <Trash2 className="w-3.5 h-3.5" />
                Trash
              </button>
            </div>

            <div className="flex items-center gap-2">
              {tab === 'files' && (
                <form
                  onSubmit={(e) => {
                    e.preventDefault();
                    setActiveSearch(searchInput.trim());
                  }}
                  className="flex items-center gap-1.5"
                >
                  <div className="relative">
                    <Search className="w-3.5 h-3.5 text-[#78716C] absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
                    <input
                      type="search"
                      value={searchInput}
                      onChange={(e) => setSearchInput(e.target.value)}
                      placeholder="Search Drive files..."
                      className="pl-8 pr-3 py-1.5 text-xs rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] focus:outline-none focus:border-[#1D4ED8]"
                    />
                  </div>
                  <button
                    type="submit"
                    className="px-3 py-1.5 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D]"
                  >
                    Search
                  </button>
                  {activeSearch && (
                    <button
                      type="button"
                      onClick={() => {
                        setSearchInput('');
                        setActiveSearch('');
                      }}
                      className="px-2 py-1.5 text-xs text-[#57534E] hover:text-[#1C1917] dark:hover:text-[#ECEEF2]"
                    >
                      Clear
                    </button>
                  )}
                </form>
              )}

              {tab !== 'transcript' && (
                <button
                  type="button"
                  onClick={loadDriveData}
                  disabled={loading}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D]"
                >
                  <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
                  Refresh
                </button>
              )}
            </div>
          </div>

          {tab === 'files' && !activeSearch && (
            <nav className="flex flex-wrap items-center gap-1 text-xs text-[#57534E] dark:text-[#9CA3AF]">
              {folderStack.map((crumb, idx) => (
                <span key={crumb.id} className="inline-flex items-center gap-1">
                  {idx > 0 && <ChevronRight className="w-3.5 h-3.5 opacity-60" />}
                  <button
                    type="button"
                    onClick={() => setFolderStack((prev) => prev.slice(0, idx + 1))}
                    className={`px-1.5 py-0.5 rounded hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] ${
                      idx === folderStack.length - 1
                        ? 'font-semibold text-[#1C1917] dark:text-[#ECEEF2]'
                        : ''
                    }`}
                  >
                    {crumb.name}
                  </button>
                </span>
              ))}
            </nav>
          )}

          {newFolderDraft !== null && (
            <form
              onSubmit={(e) => {
                e.preventDefault();
                const trimmed = newFolderDraft.trim();
                if (!trimmed) return;
                setNewFolderDraft(null);
                setPendingMutation({
                  kind: 'create_folder',
                  folderName: trimmed,
                  parentId: currentFolder.id,
                });
              }}
              className="flex flex-wrap items-center gap-2 rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/60 dark:bg-[#171A21] p-3"
            >
              <input
                type="text"
                value={newFolderDraft}
                onChange={(e) => setNewFolderDraft(e.target.value)}
                placeholder="New folder name..."
                autoFocus
                className="flex-1 min-w-[200px] px-3 py-1.5 text-sm rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317]"
              />
              <button
                type="submit"
                className="px-3 py-1.5 text-xs font-medium rounded bg-[#1D4ED8] text-white hover:bg-[#1E40AF]"
              >
                Continue
              </button>
              <button
                type="button"
                onClick={() => setNewFolderDraft(null)}
                className="px-3 py-1.5 text-xs rounded border border-[#E5E0D5] dark:border-[#262B36]"
              >
                Cancel
              </button>
            </form>
          )}

          {renameDraft !== null && (
            <form
              onSubmit={(e) => {
                e.preventDefault();
                const trimmed = renameDraft.value.trim();
                if (!trimmed || trimmed === renameDraft.file.name) return;
                const target = renameDraft.file;
                setRenameDraft(null);
                setPendingMutation({
                  kind: 'rename',
                  file: target,
                  newName: trimmed,
                });
              }}
              className="flex flex-wrap items-center gap-2 rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/60 dark:bg-[#171A21] p-3"
            >
              <span className="text-xs text-[#57534E] dark:text-[#9CA3AF]">
                Rename "{renameDraft.file.name}":
              </span>
              <input
                type="text"
                value={renameDraft.value}
                onChange={(e) =>
                  setRenameDraft({ file: renameDraft.file, value: e.target.value })
                }
                autoFocus
                className="flex-1 min-w-[200px] px-3 py-1.5 text-sm rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317]"
              />
              <button
                type="submit"
                className="px-3 py-1.5 text-xs font-medium rounded bg-[#1D4ED8] text-white hover:bg-[#1E40AF]"
              >
                Continue
              </button>
              <button
                type="button"
                onClick={() => setRenameDraft(null)}
                className="px-3 py-1.5 text-xs rounded border border-[#E5E0D5] dark:border-[#262B36]"
              >
                Cancel
              </button>
            </form>
          )}

          {tab === 'transcript' ? (
            <form
              onSubmit={(e) => {
                e.preventDefault();
                const cleanedFileName = normalizeTranscriptFileName(transcriptFileName);
                const formatted = formatTranscriptMarkdown(transcriptText, cleanedFileName);
                if (!formatted) {
                  setError('Paste chat text or choose a transcript file first.');
                  return;
                }
                setPendingMutation({
                  kind: 'upload_transcript',
                  fileName: cleanedFileName,
                  content: formatted,
                  useTranscriptSubfolder,
                  parentId: currentFolder.id,
                  parentFolderLabel: currentFolder.name,
                });
              }}
              className="rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/40 dark:bg-[#171A21] p-6 space-y-4"
            >
              <div className="space-y-1">
                <h2 className="font-display text-lg font-semibold">
                  Save Chat Transcript to Google Drive
                </h2>
                <p className="text-sm text-[#57534E] dark:text-[#9CA3AF]">
                  Paste conversation text copied from Google AI Studio or upload a saved `.txt`,
                  `.md`, or `.json` export. It will be formatted cleanly into Markdown and stored
                  in your Google Drive.
                </p>
              </div>

              <div className="flex flex-wrap items-end gap-4">
                <div className="flex-1 min-w-[220px] space-y-1.5">
                  <label
                    htmlFor="transcript-filename"
                    className="block text-xs font-medium text-[#44403C] dark:text-[#D1D5DB]"
                  >
                    File Name in Google Drive
                  </label>
                  <input
                    id="transcript-filename"
                    type="text"
                    value={transcriptFileName}
                    onChange={(e) => setTranscriptFileName(e.target.value)}
                    placeholder="exonovel-chat-2026-11-23.md"
                    className="w-full px-3 py-2 text-sm rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] focus:outline-none focus:border-[#1D4ED8]"
                  />
                </div>

                <div>
                  <label className="inline-flex items-center gap-2 px-3.5 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] hover:bg-[#EAE5DB] dark:hover:bg-[#1E232D] cursor-pointer min-h-[38px]">
                    <FileUp className="w-3.5 h-3.5" />
                    Choose .txt / .md / .json File
                    <input
                      type="file"
                      accept=".txt,.md,.markdown,.json,text/plain,text/markdown,application/json"
                      onChange={handleTranscriptFileUpload}
                      className="sr-only"
                    />
                  </label>
                </div>
              </div>

              <div className="space-y-1.5">
                <label
                  htmlFor="transcript-body"
                  className="block text-xs font-medium text-[#44403C] dark:text-[#D1D5DB]"
                >
                  Chat Transcript Text
                </label>
                <textarea
                  id="transcript-body"
                  rows={10}
                  value={transcriptText}
                  onChange={(e) => setTranscriptText(e.target.value)}
                  placeholder="Paste your AI Studio conversation here..."
                  className="w-full px-3.5 py-2.5 text-sm rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] focus:outline-none focus:border-[#1D4ED8]"
                />
              </div>

              <div className="flex flex-wrap items-center justify-between gap-4 pt-1">
                <label className="inline-flex items-center gap-2 text-xs text-[#44403C] dark:text-[#D1D5DB] cursor-pointer">
                  <input
                    type="checkbox"
                    checked={useTranscriptSubfolder}
                    onChange={(e) => setUseTranscriptSubfolder(e.target.checked)}
                    className="rounded border-[#E5E0D5] dark:border-[#262B36]"
                  />
                  Organize inside "{currentFolder.name} / ExoNovel Chat Transcripts" folder
                </label>

                <button
                  type="submit"
                  disabled={!transcriptText.trim()}
                  className="inline-flex items-center gap-2 px-4 py-2 text-xs font-medium rounded-md bg-[#1D4ED8] text-white hover:bg-[#1E40AF] transition-colors disabled:opacity-50 min-h-[38px]"
                >
                  <Upload className="w-3.5 h-3.5" />
                  Review & Save Transcript to Drive
                </button>
              </div>
            </form>
          ) : loading ? (
            <div className="rounded-lg border border-[#E5E0D5] dark:border-[#262B36] p-8 text-center text-sm text-[#57534E] dark:text-[#9CA3AF]">
              Loading Google Drive data...
            </div>
          ) : tab === 'activity' ? (
            <div className="rounded-lg border border-[#E5E0D5] dark:border-[#262B36] divide-y divide-[#E5E0D5] dark:divide-[#262B36]">
              {activities.length === 0 ? (
                <div className="p-8 text-center text-sm text-[#57534E] dark:text-[#9CA3AF]">
                  No recent Google Drive activity recorded.
                </div>
              ) : (
                activities.map((act, idx) => {
                  const actionName = act.primaryActionDetail
                    ? Object.keys(act.primaryActionDetail)[0] || 'activity'
                    : 'activity';
                  const title =
                    act.targets?.[0]?.driveItem?.title ||
                    act.targets?.[0]?.driveItem?.name ||
                    'Drive item';
                  return (
                    <div
                      key={`${act.timestamp ?? 'act'}-${idx}`}
                      className="px-4 py-3 flex items-center justify-between gap-4 text-sm"
                    >
                      <div className="min-w-0">
                        <div className="font-medium truncate">{title}</div>
                        <div className="text-xs text-[#57534E] dark:text-[#9CA3AF] capitalize">
                          Action: {actionName}
                        </div>
                      </div>
                      <div className="text-xs text-[#57534E] dark:text-[#9CA3AF] shrink-0">
                        {formatDate(act.timestamp)}
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          ) : (
            <div className="rounded-lg border border-[#E5E0D5] dark:border-[#262B36] divide-y divide-[#E5E0D5] dark:divide-[#262B36]">
              {files.length === 0 ? (
                <div className="p-8 text-center text-sm text-[#57534E] dark:text-[#9CA3AF]">
                  {tab === 'trash'
                    ? 'Google Drive Trash is empty.'
                    : activeSearch
                      ? `No Drive files match "${activeSearch}".`
                      : 'This folder is empty.'}
                </div>
              ) : (
                files.map((file) => {
                  const isFolder = file.mimeType === 'application/vnd.google-apps.folder';
                  return (
                    <div
                      key={file.id}
                      className="px-4 py-3 flex flex-wrap items-center justify-between gap-3 hover:bg-[#F3F0EA]/50 dark:hover:bg-[#171A21]/70 transition-colors"
                    >
                      <div className="flex items-center gap-3 min-w-0 flex-1">
                        {isFolder ? (
                          <Folder className="w-4 h-4 text-[#1D4ED8] dark:text-[#60A5FA] shrink-0" />
                        ) : (
                          <FileText className="w-4 h-4 text-[#57534E] dark:text-[#9CA3AF] shrink-0" />
                        )}
                        <div className="min-w-0">
                          {isFolder && tab === 'files' ? (
                            <button
                              type="button"
                              onClick={() => {
                                setActiveSearch('');
                                setSearchInput('');
                                setFolderStack((prev) => [
                                  ...prev,
                                  { id: file.id, name: file.name },
                                ]);
                              }}
                              className="text-sm font-medium text-left hover:underline truncate block max-w-full"
                            >
                              {file.name}
                            </button>
                          ) : (
                            <div className="text-sm font-medium truncate">{file.name}</div>
                          )}
                          <div className="text-xs text-[#57534E] dark:text-[#9CA3AF]">
                            {isFolder ? 'Folder' : formatFileSize(file.size)} · Modified{' '}
                            {formatDate(file.modifiedTime)}
                          </div>
                        </div>
                      </div>

                      <div className="flex items-center gap-1.5 shrink-0">
                        {!isFolder && isPreviewableTextFile(file) && tab === 'files' && (
                          <button
                            type="button"
                            onClick={() => handlePreview(file)}
                            disabled={loadingPreviewId === file.id}
                            className="inline-flex items-center gap-1 px-2.5 py-1.5 text-xs rounded border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#EAE5DB] dark:hover:bg-[#1E232D]"
                          >
                            <Eye className="w-3.5 h-3.5" />
                            {loadingPreviewId === file.id ? 'Opening...' : 'Preview'}
                          </button>
                        )}
                        {file.webViewLink && (
                          <a
                            href={file.webViewLink}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="inline-flex items-center gap-1 px-2.5 py-1.5 text-xs rounded border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#EAE5DB] dark:hover:bg-[#1E232D]"
                          >
                            <ExternalLink className="w-3.5 h-3.5" />
                            Open
                          </a>
                        )}
                        {tab === 'files' ? (
                          <>
                            <button
                              type="button"
                              onClick={() => setRenameDraft({ file, value: file.name })}
                              title="Rename"
                              className="inline-flex items-center gap-1 px-2.5 py-1.5 text-xs rounded border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#EAE5DB] dark:hover:bg-[#1E232D]"
                            >
                              <Pencil className="w-3.5 h-3.5" />
                              Rename
                            </button>
                            <button
                              type="button"
                              onClick={() => setPendingMutation({ kind: 'trash', file })}
                              title="Move to Trash"
                              className="inline-flex items-center gap-1 px-2.5 py-1.5 text-xs rounded border border-red-200 dark:border-red-900/50 text-red-700 dark:text-red-300 hover:bg-red-50 dark:hover:bg-red-950/30"
                            >
                              <Trash2 className="w-3.5 h-3.5" />
                              Trash
                            </button>
                          </>
                        ) : (
                          <>
                            <button
                              type="button"
                              onClick={() => setPendingMutation({ kind: 'restore', file })}
                              className="inline-flex items-center gap-1 px-2.5 py-1.5 text-xs rounded border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#EAE5DB] dark:hover:bg-[#1E232D]"
                            >
                              <RotateCcw className="w-3.5 h-3.5" />
                              Restore
                            </button>
                            <button
                              type="button"
                              onClick={() =>
                                setPendingMutation({ kind: 'delete_permanently', file })
                              }
                              className="inline-flex items-center gap-1 px-2.5 py-1.5 text-xs rounded border border-red-300 dark:border-red-900/60 text-red-700 dark:text-red-300 hover:bg-red-50 dark:hover:bg-red-950/40"
                            >
                              <Trash2 className="w-3.5 h-3.5" />
                              Delete Forever
                            </button>
                          </>
                        )}
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          )}

          {previewFile && (
            <div className="rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] overflow-hidden">
              <div className="px-4 py-2.5 border-b border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/70 dark:bg-[#171A21] flex items-center justify-between gap-2">
                <span className="text-xs font-medium truncate">
                  Preview: {previewFile.file.name}
                </span>
                <button
                  type="button"
                  onClick={() => setPreviewFile(null)}
                  className="p-1 rounded hover:bg-[#EAE5DB] dark:hover:bg-[#232834]"
                  aria-label="Close preview"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
              <pre className="p-4 text-xs font-mono whitespace-pre-wrap overflow-x-auto max-h-[420px]">
                {previewFile.text}
              </pre>
            </div>
          )}
        </div>
      )}

      {pendingMutation && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
          role="dialog"
          aria-modal="true"
        >
          <div className="w-full max-w-md rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] p-6 space-y-4 shadow-lg">
            {(() => {
              const promptCopy = getMutationPromptCopy(pendingMutation);
              return (
                <>
                  <h3 className="font-display text-lg font-semibold">{promptCopy.title}</h3>
                  <p className="text-sm text-[#57534E] dark:text-[#9CA3AF]">
                    {promptCopy.detail}
                  </p>
                  {backupProgress && (
                    <p className="text-xs font-medium text-[#1D4ED8] dark:text-[#60A5FA]">
                      {backupProgress}
                    </p>
                  )}
                  <div className="flex items-center justify-end gap-2 pt-2">
                    <button
                      type="button"
                      disabled={executingMutation}
                      onClick={() => setPendingMutation(null)}
                      className="px-4 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] min-h-[38px]"
                    >
                      Cancel
                    </button>
                    <button
                      type="button"
                      disabled={executingMutation}
                      onClick={confirmAndExecuteMutation}
                      className={`px-4 py-2 text-xs font-medium rounded-md text-white min-h-[38px] ${
                        promptCopy.destructive
                          ? 'bg-red-600 hover:bg-red-700'
                          : 'bg-[#1D4ED8] hover:bg-[#1E40AF]'
                      } disabled:opacity-60`}
                    >
                      {executingMutation ? 'Working...' : promptCopy.confirmLabel}
                    </button>
                  </div>
                </>
              );
            })()}
          </div>
        </div>
      )}
    </div>
  );
}
