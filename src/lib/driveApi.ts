import { clearDriveAccessToken } from './driveAuth';

export interface DriveFile {
  id: string;
  name: string;
  mimeType: string;
  modifiedTime?: string;
  size?: string;
  webViewLink?: string;
  trashed?: boolean;
  owners?: Array<{ displayName?: string; emailAddress?: string }>;
}

export interface DriveActivityEntry {
  timestamp?: string;
  primaryActionDetail?: Record<string, unknown>;
  targets?: Array<{ driveItem?: { name?: string; title?: string; mimeType?: string } }>;
  actors?: Array<{ user?: { knownUser?: { personName?: string; isCurrentUser?: boolean } } }>;
}

export class DriveAuthExpiredError extends Error {
  constructor(message = 'Your Google Drive session expired. Please sign in with Google again.') {
    super(message);
    this.name = 'DriveAuthExpiredError';
  }
}

async function driveFetch(url: string, accessToken: string, init?: RequestInit): Promise<Response> {
  const res = await fetch(url, {
    ...init,
    headers: {
      Authorization: `Bearer ${accessToken}`,
      ...(init?.headers ?? {}),
    },
  });

  if (res.status === 401) {
    clearDriveAccessToken();
    throw new DriveAuthExpiredError();
  }

  if (!res.ok) {
    const errText = await res.text().catch(() => '');
    throw new Error(errText || `Google Drive request failed (${res.status})`);
  }

  return res;
}

export async function listDriveFiles(
  accessToken: string,
  options: {
    folderId?: string;
    searchQuery?: string;
    trashedOnly?: boolean;
    pageSize?: number;
  } = {}
): Promise<DriveFile[]> {
  const clauses: string[] = [];

  if (options.trashedOnly) {
    clauses.push('trashed = true');
  } else {
    clauses.push('trashed = false');
  }

  if (options.searchQuery && options.searchQuery.trim()) {
    const escaped = options.searchQuery.trim().replace(/\\/g, '\\\\').replace(/'/g, "\\'");
    clauses.push(`name contains '${escaped}'`);
  } else if (options.folderId && !options.trashedOnly) {
    const safeParent = options.folderId.replace(/[^a-zA-Z0-9_-]/g, '');
    clauses.push(`'${safeParent}' in parents`);
  }

  const params = new URLSearchParams({
    q: clauses.join(' and '),
    pageSize: String(options.pageSize ?? 50),
    orderBy: 'folder,modifiedTime desc',
    fields: 'files(id,name,mimeType,modifiedTime,size,webViewLink,trashed,owners)',
    supportsAllDrives: 'true',
    includeItemsFromAllDrives: 'true',
  });

  const res = await driveFetch(
    `https://www.googleapis.com/drive/v3/files?${params.toString()}`,
    accessToken
  );
  const data = (await res.json()) as { files?: DriveFile[] };
  return data.files ?? [];
}

export async function listDriveActivity(
  accessToken: string,
  pageSize = 20
): Promise<DriveActivityEntry[]> {
  const res = await driveFetch(
    'https://driveactivity.googleapis.com/v2/activity:query',
    accessToken,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pageSize }),
    }
  );
  const data = (await res.json()) as { activities?: DriveActivityEntry[] };
  return data.activities ?? [];
}

export async function readDriveFileText(accessToken: string, file: DriveFile): Promise<string> {
  if (file.mimeType === 'application/vnd.google-apps.document') {
    const res = await driveFetch(
      `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(file.id)}/export?mimeType=text/markdown`,
      accessToken
    );
    return res.text();
  }

  if (file.mimeType === 'application/vnd.google-apps.spreadsheet') {
    const res = await driveFetch(
      `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(file.id)}/export?mimeType=text/csv`,
      accessToken
    );
    return res.text();
  }

  if (file.mimeType === 'application/vnd.google-apps.presentation') {
    const res = await driveFetch(
      `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(file.id)}/export?mimeType=text/plain`,
      accessToken
    );
    return res.text();
  }

  const res = await driveFetch(
    `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(file.id)}?alt=media&supportsAllDrives=true`,
    accessToken
  );
  return res.text();
}

export async function createDriveFolder(
  accessToken: string,
  name: string,
  parentId?: string
): Promise<DriveFile> {
  const metadata: Record<string, unknown> = {
    name,
    mimeType: 'application/vnd.google-apps.folder',
  };
  if (parentId && parentId !== 'root') {
    metadata.parents = [parentId];
  }

  const res = await driveFetch(
    'https://www.googleapis.com/drive/v3/files?supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,webViewLink',
    accessToken,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(metadata),
    }
  );
  return (await res.json()) as DriveFile;
}

export function escapeDriveQueryValue(value: string): string {
  return value.replace(/\\/g, '\\\\').replace(/'/g, "\\'");
}

export async function findDriveItemByName(
  accessToken: string,
  options: {
    name: string;
    parentId?: string;
    mimeType?: string;
    excludeFolders?: boolean;
  }
): Promise<DriveFile | null> {
  const clauses: string[] = [
    'trashed = false',
    `name = '${escapeDriveQueryValue(options.name)}'`,
  ];

  const parent = (options.parentId || 'root').replace(/[^a-zA-Z0-9_-]/g, '') || 'root';
  clauses.push(`'${parent}' in parents`);

  if (options.mimeType) {
    clauses.push(`mimeType = '${escapeDriveQueryValue(options.mimeType)}'`);
  } else if (options.excludeFolders) {
    clauses.push(`mimeType != 'application/vnd.google-apps.folder'`);
  }

  const params = new URLSearchParams({
    q: clauses.join(' and '),
    pageSize: '1',
    fields: 'files(id,name,mimeType,modifiedTime,size,webViewLink,trashed)',
    supportsAllDrives: 'true',
    includeItemsFromAllDrives: 'true',
  });

  const res = await driveFetch(
    `https://www.googleapis.com/drive/v3/files?${params.toString()}`,
    accessToken
  );
  const data = (await res.json()) as { files?: DriveFile[] };
  return data.files?.[0] ?? null;
}

export async function ensureDriveFolder(
  accessToken: string,
  name: string,
  parentId = 'root'
): Promise<{ folder: DriveFile; created: boolean }> {
  const existing = await findDriveItemByName(accessToken, {
    name,
    parentId,
    mimeType: 'application/vnd.google-apps.folder',
  });
  if (existing) {
    return { folder: existing, created: false };
  }
  const folder = await createDriveFolder(accessToken, name, parentId);
  return { folder, created: true };
}

export async function uploadMarkdownToDrive(
  accessToken: string,
  options: {
    name: string;
    content: string;
    mimeType?: string;
    parentId?: string;
  }
): Promise<DriveFile> {
  const metadata: Record<string, unknown> = {
    name: options.name,
    mimeType: options.mimeType ?? 'text/markdown',
  };
  if (options.parentId && options.parentId !== 'root') {
    metadata.parents = [options.parentId];
  }

  const boundary = `exonovel_drive_${crypto.randomUUID()}`;
  const multipartBody =
    `--${boundary}\r\n` +
    `Content-Type: application/json; charset=UTF-8\r\n\r\n` +
    `${JSON.stringify(metadata)}\r\n` +
    `--${boundary}\r\n` +
    `Content-Type: ${options.mimeType ?? 'text/markdown'}; charset=UTF-8\r\n\r\n` +
    `${options.content}\r\n` +
    `--${boundary}--`;

  const res = await driveFetch(
    'https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,size,webViewLink',
    accessToken,
    {
      method: 'POST',
      headers: {
        'Content-Type': `multipart/related; boundary=${boundary}`,
      },
      body: multipartBody,
    }
  );
  return (await res.json()) as DriveFile;
}

export async function updateDriveTextFile(
  accessToken: string,
  fileId: string,
  options: {
    name?: string;
    content: string;
    mimeType?: string;
  }
): Promise<DriveFile> {
  const metadata: Record<string, unknown> = {};
  if (options.name) {
    metadata.name = options.name;
  }

  const boundary = `exonovel_drive_${crypto.randomUUID()}`;
  const multipartBody =
    `--${boundary}\r\n` +
    `Content-Type: application/json; charset=UTF-8\r\n\r\n` +
    `${JSON.stringify(metadata)}\r\n` +
    `--${boundary}\r\n` +
    `Content-Type: ${options.mimeType ?? 'text/markdown'}; charset=UTF-8\r\n\r\n` +
    `${options.content}\r\n` +
    `--${boundary}--`;

  const res = await driveFetch(
    `https://www.googleapis.com/upload/drive/v3/files/${encodeURIComponent(fileId)}?uploadType=multipart&supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,size,webViewLink`,
    accessToken,
    {
      method: 'PATCH',
      headers: {
        'Content-Type': `multipart/related; boundary=${boundary}`,
      },
      body: multipartBody,
    }
  );
  return (await res.json()) as DriveFile;
}

export async function upsertMarkdownFileInDrive(
  accessToken: string,
  options: {
    name: string;
    content: string;
    mimeType?: string;
    parentId?: string;
  }
): Promise<{ file: DriveFile; action: 'created' | 'updated' }> {
  const existing = await findDriveItemByName(accessToken, {
    name: options.name,
    parentId: options.parentId,
    excludeFolders: true,
  });

  if (existing) {
    const file = await updateDriveTextFile(accessToken, existing.id, {
      name: options.name,
      content: options.content,
      mimeType: options.mimeType,
    });
    return { file, action: 'updated' };
  }

  const file = await uploadMarkdownToDrive(accessToken, options);
  return { file, action: 'created' };
}

export async function backupDocsArchiveToDrive(
  accessToken: string,
  options: {
    rootParentId: string;
    rootFolderName: string;
    docs: Array<{ path: string; content: string }>;
    onProgress?: (done: number, total: number, currentPath: string) => void;
  }
): Promise<{
  archiveFolder: DriveFile;
  createdCount: number;
  updatedCount: number;
  total: number;
}> {
  const { folder: archiveFolder } = await ensureDriveFolder(
    accessToken,
    options.rootFolderName,
    options.rootParentId
  );

  const folderCache = new Map<string, string>();
  folderCache.set('', archiveFolder.id);

  let createdCount = 0;
  let updatedCount = 0;
  const total = options.docs.length;

  for (let i = 0; i < total; i += 1) {
    const doc = options.docs[i];
    const normalized = doc.path.replace(/^docs\//i, '');
    const parts = normalized.split('/').filter(Boolean);
    const fileName = parts.pop() || 'document.md';

    let currentParentId = archiveFolder.id;
    let cacheKey = '';
    for (const segment of parts) {
      cacheKey = cacheKey ? `${cacheKey}/${segment}` : segment;
      const cachedId = folderCache.get(cacheKey);
      if (cachedId) {
        currentParentId = cachedId;
      } else {
        const ensured = await ensureDriveFolder(accessToken, segment, currentParentId);
        currentParentId = ensured.folder.id;
        folderCache.set(cacheKey, currentParentId);
      }
    }

    const result = await upsertMarkdownFileInDrive(accessToken, {
      name: fileName,
      content: doc.content,
      mimeType: 'text/markdown',
      parentId: currentParentId,
    });

    if (result.action === 'created') {
      createdCount += 1;
    } else {
      updatedCount += 1;
    }

    options.onProgress?.(i + 1, total, doc.path);
  }

  return {
    archiveFolder,
    createdCount,
    updatedCount,
    total,
  };
}

export async function renameDriveFile(
  accessToken: string,
  fileId: string,
  newName: string
): Promise<DriveFile> {
  const res = await driveFetch(
    `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(fileId)}?supportsAllDrives=true&fields=id,name,mimeType,modifiedTime,webViewLink`,
    accessToken,
    {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: newName }),
    }
  );
  return (await res.json()) as DriveFile;
}

export async function setDriveFileTrashed(
  accessToken: string,
  fileId: string,
  trashed: boolean
): Promise<DriveFile> {
  const res = await driveFetch(
    `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(fileId)}?supportsAllDrives=true&fields=id,name,trashed`,
    accessToken,
    {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ trashed }),
    }
  );
  return (await res.json()) as DriveFile;
}

export async function deleteDriveFilePermanently(
  accessToken: string,
  fileId: string
): Promise<void> {
  await driveFetch(
    `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(fileId)}?supportsAllDrives=true`,
    accessToken,
    {
      method: 'DELETE',
    }
  );
}
