import { useCallback, useEffect, useState, type FormEvent } from 'react';
import {
  ArrowLeft,
  Archive,
  CheckCircle2,
  ExternalLink,
  HardDrive,
  Plus,
  ShieldOff,
  Trash2,
} from 'lucide-react';
import {
  getDriveAccessToken,
  getDriveUserEmail,
  googleDriveSignIn,
  revokeGoogleDriveAccess,
} from '../lib/driveAuth';

interface AllowlistEntry {
  email: string;
  enabled: boolean;
  role: string;
  note: string;
  addedBy: string;
  updatedAt: string;
}

interface IdentityRow {
  didPeer: string;
  email: string | null;
  authUserId: string;
  provider: string;
  createdAt: string;
}

interface SnapshotRow {
  id: number;
  docPath: string;
  objectKey: string;
  sizeBytes: number;
  uploadedByDid: string;
  createdAt: string;
  downloadUrl: string;
}

export interface LinkedProviderSummary {
  providerId: string;
  scopes: string[];
}

interface AdminSettingsProps {
  getToken: () => Promise<string | null>;
  activeDocPath: string;
  currentUserDid: string | null;
  currentUserEmail: string | null;
  isAdmin: boolean;
  linkedProviders: LinkedProviderSummary[];
  driveConnected: boolean;
  onDriveAuthChange: (connected: boolean) => void;
  onOpenDrive: () => void;
  onBackToDocs: () => void;
}

export function AdminSettings({
  getToken,
  activeDocPath,
  currentUserDid,
  currentUserEmail,
  isAdmin,
  linkedProviders,
  driveConnected,
  onDriveAuthChange,
  onOpenDrive,
  onBackToDocs,
}: AdminSettingsProps) {
  const [entries, setEntries] = useState<AllowlistEntry[]>([]);
  const [identities, setIdentities] = useState<IdentityRow[]>([]);
  const [snapshots, setSnapshots] = useState<SnapshotRow[]>([]);
  const [storageReady, setStorageReady] = useState(true);
  const [newEmail, setNewEmail] = useState('');
  const [newNote, setNewNote] = useState('');
  const [loading, setLoading] = useState(isAdmin);
  const [saving, setSaving] = useState(false);
  const [archiving, setArchiving] = useState(false);
  const [driveActionLoading, setDriveActionLoading] = useState(false);
  const [confirmRevokeDrive, setConfirmRevokeDrive] = useState(false);
  const [driveNotice, setDriveNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const authFetch = useCallback(
    async (url: string, init?: RequestInit) => {
      const token = await getToken();
      return fetch(url, {
        ...init,
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
          ...(init?.headers || {}),
        },
      });
    },
    [getToken]
  );

  const loadData = useCallback(async () => {
    if (!isAdmin) {
      setLoading(false);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const [allowRes, idRes, snapRes] = await Promise.all([
        authFetch('/api/admin/allowlist'),
        authFetch('/api/admin/identities'),
        authFetch('/api/storage/snapshots'),
      ]);

      if (!allowRes.ok) {
        const body = await allowRes.json().catch(() => ({}));
        throw new Error(body.error || 'Failed to load allowlist.');
      }

      const allowData = (await allowRes.json()) as { entries: AllowlistEntry[] };
      setEntries(allowData.entries);

      if (idRes.ok) {
        const idData = (await idRes.json()) as { identities: IdentityRow[] };
        setIdentities(idData.identities);
      }

      if (snapRes.ok) {
        const snapData = (await snapRes.json()) as { snapshots: SnapshotRow[] };
        setSnapshots(snapData.snapshots);
        setStorageReady(true);
      } else if (snapRes.status === 503) {
        setStorageReady(false);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to load settings.');
    } finally {
      setLoading(false);
    }
  }, [authFetch, isAdmin]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const googleProvider = linkedProviders.find((p) => p.providerId === 'google');
  const isDriveActive = driveConnected || Boolean(getDriveAccessToken());
  const activeDriveEmail = getDriveUserEmail();

  const handleConnectDrive = async () => {
    setDriveActionLoading(true);
    setError(null);
    setDriveNotice(null);
    try {
      const { user } = await googleDriveSignIn({
        loginHint: googleProvider && currentUserEmail ? currentUserEmail : undefined,
      });
      onDriveAuthChange(true);
      setDriveNotice(
        `Connected Google Drive as ${user.email || currentUserEmail || 'Google User'}.`
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Could not connect Google Drive.');
    } finally {
      setDriveActionLoading(false);
    }
  };

  const handleConfirmRevokeDrive = async () => {
    setDriveActionLoading(true);
    setError(null);
    setDriveNotice(null);
    try {
      await revokeGoogleDriveAccess(getToken);
      onDriveAuthChange(false);
      setConfirmRevokeDrive(false);
      setDriveNotice(
        'Google Drive access token was revoked and disconnected. Your ExoNovel sign-in remains active.'
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke Google Drive access.');
    } finally {
      setDriveActionLoading(false);
    }
  };

  const handleAddUser = async (e: FormEvent) => {
    e.preventDefault();
    if (!newEmail.trim()) return;
    setSaving(true);
    setError(null);

    try {
      const res = await authFetch('/api/admin/allowlist', {
        method: 'POST',
        body: JSON.stringify({
          email: newEmail.trim(),
          enabled: true,
          note: newNote.trim(),
        }),
      });
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error || 'Failed to add user.');
      }
      setNewEmail('');
      setNewNote('');
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update access list.');
    } finally {
      setSaving(false);
    }
  };

  const handleToggleEnabled = async (entry: AllowlistEntry) => {
    setError(null);
    try {
      const res = await authFetch('/api/admin/allowlist', {
        method: 'POST',
        body: JSON.stringify({
          email: entry.email,
          enabled: !entry.enabled,
          note: entry.note,
        }),
      });
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error || 'Failed to toggle user access.');
      }
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to update entry.');
    }
  };

  const handleDelete = async (email: string) => {
    setError(null);
    try {
      const res = await authFetch(
        `/api/admin/allowlist?email=${encodeURIComponent(email)}`,
        { method: 'DELETE' }
      );
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error || 'Failed to remove entry.');
      }
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to remove entry.');
    }
  };

  const handleSnapshot = async () => {
    if (!activeDocPath) return;
    setArchiving(true);
    setError(null);
    try {
      const res = await authFetch('/api/storage/snapshot', {
        method: 'POST',
        body: JSON.stringify({ docPath: activeDocPath }),
      });
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error || 'Failed to archive snapshot.');
      }
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Snapshot upload failed.');
    } finally {
      setArchiving(false);
    }
  };

  return (
    <div className="w-full">
      <div className="flex items-center justify-between gap-4 pb-6 border-b border-[#E5E0D5] dark:border-[#262B36]">
        <div>
          <h1 className="font-display text-3xl font-semibold tracking-tight">
            {isAdmin ? 'Settings & Admin Controls' : 'Account & Connected Apps Settings'}
          </h1>
          <p className="mt-1.5 text-sm text-[#57534E] dark:text-[#9CA3AF]">
            {isAdmin
              ? 'Manage connected external apps, private documentation access, resolved did:peer identities, and Neon Object Storage snapshots.'
              : 'Manage your linked sign-in providers and connected external apps.'}
          </p>
        </div>

        <button
          type="button"
          onClick={onBackToDocs}
          className="inline-flex items-center gap-2 px-4 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors whitespace-nowrap shrink-0 min-h-[40px]"
        >
          <ArrowLeft className="w-3.5 h-3.5" />
          <span>Back to Docs</span>
        </button>
      </div>

      {currentUserDid && (
        <div className="mt-6 p-4 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/60 dark:bg-[#171A21]">
          <div className="text-xs text-[#57534E] dark:text-[#9CA3AF]">
            Signed in as {currentUserEmail} · Authoritative Owner ID
          </div>
          <div className="mt-1 font-mono text-xs break-all text-[#1C1917] dark:text-[#ECEEF2]">
            {currentUserDid}
          </div>
        </div>
      )}

      {error && (
        <div className="mt-6 p-3.5 rounded-lg border border-red-300 dark:border-red-900/60 bg-red-50 dark:bg-red-950/40 text-xs text-red-700 dark:text-red-300">
          {error}
        </div>
      )}

      {driveNotice && (
        <div className="mt-6 p-3.5 rounded-lg border border-emerald-300 dark:border-emerald-900/60 bg-emerald-50 dark:bg-emerald-950/30 text-xs text-emerald-800 dark:text-emerald-200">
          {driveNotice}
        </div>
      )}

      <section className="mt-8">
        <h2 className="font-display text-xl font-semibold">
          Connected Apps & SSO Permissions
        </h2>
        <p className="mt-1 text-sm text-[#57534E] dark:text-[#9CA3AF]">
          Your ExoNovel sign-in only requests basic identity (
          <code className="font-mono text-xs">email</code>,{' '}
          <code className="font-mono text-xs">profile</code>). External apps below are strictly
          optional and can be connected or revoked here without affecting your ExoNovel account.
        </p>

        <div className="mt-4 grid gap-4 md:grid-cols-2">
          <div className="rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/40 dark:bg-[#171A21] p-5 space-y-3">
            <div className="flex items-center justify-between gap-2">
              <div className="font-medium text-sm">ExoNovel Sign-In Providers (Neon Auth)</div>
              <span className="text-xs px-2 py-0.5 rounded bg-[#EAE5DB] dark:bg-[#232834] text-[#44403C] dark:text-[#D1D5DB]">
                {linkedProviders.length > 0 ? `${linkedProviders.length} linked` : 'Session active'}
              </span>
            </div>
            {linkedProviders.length === 0 ? (
              <p className="text-xs text-[#57534E] dark:text-[#9CA3AF]">
                Signed in via session ({currentUserEmail || 'verified user'}).
              </p>
            ) : (
              <ul className="space-y-2 text-xs">
                {linkedProviders.map((prov) => (
                  <li
                    key={prov.providerId}
                    className="rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] px-3 py-2"
                  >
                    <div className="font-medium capitalize">{prov.providerId}</div>
                    <div className="mt-0.5 text-[#57534E] dark:text-[#9CA3AF]">
                      Login scopes:{' '}
                      {prov.scopes.length > 0
                        ? prov.scopes
                            .map((s) => s.replace('https://www.googleapis.com/auth/', ''))
                            .join(', ')
                        : 'identity only'}
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/40 dark:bg-[#171A21] p-5 flex flex-col justify-between gap-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between gap-2">
                <div className="inline-flex items-center gap-2 font-medium text-sm">
                  <HardDrive className="w-4 h-4 text-[#1D4ED8] dark:text-[#60A5FA]" />
                  <span>Google Drive</span>
                </div>
                <span
                  className={`inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded ${
                    isDriveActive
                      ? 'bg-emerald-100 dark:bg-emerald-950/50 text-emerald-800 dark:text-emerald-300'
                      : 'bg-[#EAE5DB] dark:bg-[#232834] text-[#57534E] dark:text-[#9CA3AF]'
                  }`}
                >
                  {isDriveActive && <CheckCircle2 className="w-3 h-3" />}
                  {isDriveActive ? 'Connected (Session)' : 'Not Connected'}
                </span>
              </div>
              <p className="text-xs text-[#57534E] dark:text-[#9CA3AF] leading-relaxed">
                {isDriveActive
                  ? `Connected as ${activeDriveEmail || currentUserEmail || 'Google User'}. Tokens remain in active browser memory for security in this preview environment.`
                  : 'Connect Google Drive to back up docs and save chat transcripts directly to Drive. When deployed to production exonovel.exosystems.net, encrypted offline grants stay connected across reloads until revoked here.'}
              </p>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              {isDriveActive ? (
                <>
                  <button
                    type="button"
                    onClick={onOpenDrive}
                    className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] hover:bg-[#EAE5DB] dark:hover:bg-[#1E232D] min-h-[38px]"
                  >
                    <HardDrive className="w-3.5 h-3.5" />
                    Open Google Drive
                  </button>
                  <button
                    type="button"
                    disabled={driveActionLoading}
                    onClick={() => setConfirmRevokeDrive(true)}
                    className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md border border-red-300 dark:border-red-900/60 text-red-700 dark:text-red-300 hover:bg-red-50 dark:hover:bg-red-950/30 min-h-[38px] disabled:opacity-50"
                  >
                    <ShieldOff className="w-3.5 h-3.5" />
                    Disconnect & Revoke
                  </button>
                </>
              ) : (
                <button
                  type="button"
                  disabled={driveActionLoading}
                  onClick={handleConnectDrive}
                  className="inline-flex items-center gap-1.5 px-3.5 py-2 text-xs font-medium rounded-md bg-[#1D4ED8] text-white hover:bg-[#1E40AF] transition-colors min-h-[38px] disabled:opacity-50"
                >
                  <HardDrive className="w-3.5 h-3.5" />
                  {driveActionLoading ? 'Connecting...' : 'Connect Google Drive'}
                </button>
              )}
            </div>
          </div>
        </div>
      </section>

      {isAdmin && (
        <>
          <section className="mt-10">
            <h2 className="font-display text-xl font-semibold">
              Documentation Access Allowlist
            </h2>
        <p className="mt-1 text-sm text-[#57534E] dark:text-[#9CA3AF]">
          Only email addresses enabled below can read ExoNovel documentation.
        </p>

        <form onSubmit={handleAddUser} className="mt-4 flex flex-col sm:flex-row gap-3">
          <input
            type="email"
            required
            value={newEmail}
            onChange={(e) => setNewEmail(e.target.value)}
            placeholder="collaborator@example.com"
            aria-label="Email to grant docs access"
            className="flex-1 px-3.5 py-2 text-sm rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] focus:outline-none focus:border-[#1D4ED8] dark:focus:border-[#60A5FA]"
          />
          <input
            type="text"
            value={newNote}
            onChange={(e) => setNewNote(e.target.value)}
            placeholder="Role or note (optional)"
            aria-label="Optional note"
            className="sm:w-52 px-3.5 py-2 text-sm rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] focus:outline-none focus:border-[#1D4ED8] dark:focus:border-[#60A5FA]"
          />
          <button
            type="submit"
            disabled={saving}
            className="inline-flex items-center justify-center gap-2 px-4 py-2 text-xs font-medium rounded-md bg-[#1D4ED8] text-white hover:bg-[#1E40AF] transition-colors whitespace-nowrap shrink-0 min-h-[40px] disabled:opacity-50"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>{saving ? 'Saving...' : 'Grant Access'}</span>
          </button>
        </form>

        <div className="mt-4 overflow-x-auto rounded-lg border border-[#E5E0D5] dark:border-[#262B36]">
          <table className="w-full text-left border-collapse text-sm tabular-nums">
            <thead className="bg-[#F3F0EA] dark:bg-[#171A21] border-b border-[#E5E0D5] dark:border-[#262B36]">
              <tr>
                <th className="px-4 py-2.5 font-semibold">Email</th>
                <th className="px-4 py-2.5 font-semibold">Role</th>
                <th className="px-4 py-2.5 font-semibold">Status</th>
                <th className="px-4 py-2.5 font-semibold">Note</th>
                <th className="px-4 py-2.5 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr>
                  <td colSpan={5} className="px-4 py-6 text-center text-xs text-[#57534E]">
                    Loading allowlist...
                  </td>
                </tr>
              ) : (
                entries.map((entry) => (
                  <tr
                    key={entry.email}
                    className="border-t border-[#E5E0D5] dark:border-[#262B36]"
                  >
                    <td className="px-4 py-2.5 font-medium">{entry.email}</td>
                    <td className="px-4 py-2.5 text-xs text-[#57534E] dark:text-[#9CA3AF]">
                      {entry.role}
                    </td>
                    <td className="px-4 py-2.5 text-xs">
                      {entry.enabled ? 'Allowed' : 'Paused'}
                    </td>
                    <td className="px-4 py-2.5 text-xs text-[#57534E] dark:text-[#9CA3AF]">
                      {entry.note || '—'}
                    </td>
                    <td className="px-4 py-2.5 text-right">
                      {entry.role !== 'admin' ? (
                        <div className="inline-flex items-center gap-2">
                          <button
                            type="button"
                            onClick={() => handleToggleEnabled(entry)}
                            className="px-2.5 py-1 text-xs rounded border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] whitespace-nowrap"
                          >
                            {entry.enabled ? 'Disable' : 'Enable'}
                          </button>
                          <button
                            type="button"
                            onClick={() => handleDelete(entry.email)}
                            className="p-1.5 rounded text-[#57534E] hover:text-red-600 dark:hover:text-red-400"
                            aria-label={`Remove ${entry.email}`}
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      ) : (
                        <span className="text-xs text-[#78716C]">Primary Admin</span>
                      )}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </section>

      <section className="mt-10">
        <h2 className="font-display text-xl font-semibold">
          Resolved <span className="font-mono text-base">did:peer</span> Identities
        </h2>
        <div className="mt-4 overflow-x-auto rounded-lg border border-[#E5E0D5] dark:border-[#262B36]">
          <table className="w-full text-left border-collapse text-sm tabular-nums">
            <thead className="bg-[#F3F0EA] dark:bg-[#171A21] border-b border-[#E5E0D5] dark:border-[#262B36]">
              <tr>
                <th className="px-4 py-2.5 font-semibold">Email</th>
                <th className="px-4 py-2.5 font-semibold">did:peer</th>
                <th className="px-4 py-2.5 font-semibold">Provider</th>
              </tr>
            </thead>
            <tbody>
              {identities.length === 0 ? (
                <tr>
                  <td colSpan={3} className="px-4 py-5 text-center text-xs text-[#57534E]">
                    No identities recorded yet.
                  </td>
                </tr>
              ) : (
                identities.map((id) => (
                  <tr key={id.didPeer} className="border-t border-[#E5E0D5] dark:border-[#262B36]">
                    <td className="px-4 py-2.5">{id.email || '—'}</td>
                    <td className="px-4 py-2.5 font-mono text-xs break-all">{id.didPeer}</td>
                    <td className="px-4 py-2.5 text-xs text-[#57534E] dark:text-[#9CA3AF]">
                      {id.provider}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </section>

      <section className="mt-10 pb-8">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h2 className="font-display text-xl font-semibold">
              Neon Object Storage Snapshots
            </h2>
            <p className="mt-1 text-sm text-[#57534E] dark:text-[#9CA3AF]">
              Archive Markdown snapshots to the private <code className="font-mono text-xs">assets</code> bucket.
            </p>
          </div>
          {storageReady && activeDocPath && (
            <button
              type="button"
              onClick={handleSnapshot}
              disabled={archiving}
              className="inline-flex items-center gap-2 px-4 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors whitespace-nowrap shrink-0 min-h-[40px] disabled:opacity-50"
            >
              <Archive className="w-3.5 h-3.5" />
              <span>{archiving ? 'Archiving...' : `Archive ${activeDocPath}`}</span>
            </button>
          )}
        </div>

        {!storageReady ? (
          <div className="mt-4 p-4 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] text-xs text-[#57534E] dark:text-[#9CA3AF]">
            Neon Object Storage credentials are not active in this environment yet.
          </div>
        ) : (
          <div className="mt-4 overflow-x-auto rounded-lg border border-[#E5E0D5] dark:border-[#262B36]">
            <table className="w-full text-left border-collapse text-sm tabular-nums">
              <thead className="bg-[#F3F0EA] dark:bg-[#171A21] border-b border-[#E5E0D5] dark:border-[#262B36]">
                <tr>
                  <th className="px-4 py-2.5 font-semibold">Document</th>
                  <th className="px-4 py-2.5 font-semibold">Object Key</th>
                  <th className="px-4 py-2.5 font-semibold text-right">Size</th>
                  <th className="px-4 py-2.5 font-semibold text-right">Download</th>
                </tr>
              </thead>
              <tbody>
                {snapshots.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="px-4 py-5 text-center text-xs text-[#57534E]">
                      No archived snapshots yet.
                    </td>
                  </tr>
                ) : (
                  snapshots.map((snap) => (
                    <tr key={snap.id} className="border-t border-[#E5E0D5] dark:border-[#262B36]">
                      <td className="px-4 py-2.5 font-mono text-xs">{snap.docPath}</td>
                      <td className="px-4 py-2.5 font-mono text-xs text-[#57534E] dark:text-[#9CA3AF]">
                        {snap.objectKey}
                      </td>
                      <td className="px-4 py-2.5 text-right text-xs">{snap.sizeBytes} B</td>
                      <td className="px-4 py-2.5 text-right">
                        <a
                          href={snap.downloadUrl}
                          target="_blank"
                          rel="noreferrer"
                          className="inline-flex items-center gap-1 text-xs text-[#1D4ED8] dark:text-[#60A5FA] hover:underline"
                        >
                          <span>Presigned URL</span>
                          <ExternalLink className="w-3 h-3" />
                        </a>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        )}
      </section>
        </>
      )}

      {confirmRevokeDrive && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
          role="dialog"
          aria-modal="true"
        >
          <div className="w-full max-w-md rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#171A21] p-6 space-y-4 shadow-lg">
            <h3 className="font-display text-lg font-semibold">
              Disconnect & revoke Google Drive access?
            </h3>
            <p className="text-sm text-[#57534E] dark:text-[#9CA3AF]">
              This revokes ExoNovel&apos;s Google Drive access token with Google and disconnects
              Google Drive from this session. Your ExoNovel sign-in remains active.
            </p>
            <div className="flex items-center justify-end gap-2 pt-2">
              <button
                type="button"
                disabled={driveActionLoading}
                onClick={() => setConfirmRevokeDrive(false)}
                className="px-4 py-2 text-xs font-medium rounded-md border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] min-h-[38px]"
              >
                Cancel
              </button>
              <button
                type="button"
                disabled={driveActionLoading}
                onClick={handleConfirmRevokeDrive}
                className="px-4 py-2 text-xs font-medium rounded-md bg-red-600 hover:bg-red-700 text-white min-h-[38px] disabled:opacity-60"
              >
                {driveActionLoading ? 'Revoking...' : 'Revoke Access'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
