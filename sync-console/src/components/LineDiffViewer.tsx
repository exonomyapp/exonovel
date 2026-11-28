import { useMemo, useState } from 'react';
import { X } from 'lucide-react';

export interface DiffHunkLine {
  type: 'ctx' | 'add' | 'del';
  oldLine: number | null;
  newLine: number | null;
  text: string;
}

export interface WorkspaceFileDiffDetail {
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
  diffHunks: DiffHunkLine[];
  diffTruncated: boolean;
  truncated: boolean;
}

interface LineDiffViewerProps {
  previewPath: string;
  previewLoading: boolean;
  previewData: WorkspaceFileDiffDetail | null;
  onClose: () => void;
}

export function LineDiffViewer({
  previewPath,
  previewLoading,
  previewData,
  onClose,
}: LineDiffViewerProps) {
  const [viewMode, setViewMode] = useState<'line' | 'side'>('line');
  const [changesOnly, setChangesOnly] = useState(true);

  const visibleHunks = useMemo(() => {
    if (!previewData) return [];
    if (!changesOnly) return previewData.diffHunks;

    const hunks = previewData.diffHunks;
    const include = new Set<number>();
    const contextLines = 3;
    for (let i = 0; i < hunks.length; i += 1) {
      if (hunks[i].type !== 'ctx') {
        for (
          let k = Math.max(0, i - contextLines);
          k <= Math.min(hunks.length - 1, i + contextLines);
          k += 1
        ) {
          include.add(k);
        }
      }
    }
    if (include.size === 0) return hunks;
    return hunks.filter((_, idx) => include.has(idx));
  }, [previewData, changesOnly]);

  const changeCounts = useMemo(() => {
    if (!previewData) return { added: 0, removed: 0 };
    let added = 0;
    let removed = 0;
    for (const h of previewData.diffHunks) {
      if (h.type === 'add') added += 1;
      else if (h.type === 'del') removed += 1;
    }
    return { added, removed };
  }, [previewData]);

  return (
    <div className="rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] p-4 space-y-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <div className="font-mono text-xs font-medium truncate">{previewPath}</div>
          {previewData && !previewData.isBinary && (
            <div className="mt-0.5 text-[11px] font-mono tabular-nums text-[#57534E] dark:text-[#9CA3AF]">
              <span className="text-emerald-700 dark:text-emerald-400">
                +{changeCounts.added}
              </span>
              {' · '}
              <span className="text-red-700 dark:text-red-400">
                -{changeCounts.removed}
              </span>
              {' · '}
              <span>
                remote {previewData.existsRemote ? `${previewData.remoteSize} B` : 'absent'}
              </span>
              {' · '}
              <span>
                local {previewData.existsLocal ? `${previewData.localSize} B` : 'absent'}
              </span>
            </div>
          )}
        </div>

        <div className="flex items-center gap-2 shrink-0">
          {previewData && !previewData.isBinary && (
            <>
              <div className="inline-flex items-center p-0.5 rounded-md border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] text-[11px]">
                <button
                  type="button"
                  onClick={() => setViewMode('line')}
                  className={`px-2 py-1 rounded transition-colors whitespace-nowrap ${
                    viewMode === 'line'
                      ? 'bg-[#1D4ED8] text-white font-medium'
                      : 'text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2]'
                  }`}
                >
                  Line-by-Line
                </button>
                <button
                  type="button"
                  onClick={() => setViewMode('side')}
                  className={`px-2 py-1 rounded transition-colors whitespace-nowrap ${
                    viewMode === 'side'
                      ? 'bg-[#1D4ED8] text-white font-medium'
                      : 'text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2]'
                  }`}
                >
                  Side-by-Side
                </button>
              </div>

              {viewMode === 'line' && (
                <button
                  type="button"
                  onClick={() => setChangesOnly((prev) => !prev)}
                  className="px-2.5 py-1 rounded border border-[#E5E0D5] dark:border-[#262B36] text-[11px] text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2] whitespace-nowrap"
                >
                  {changesOnly ? 'Show Full File' : 'Changes Only'}
                </button>
              )}
            </>
          )}

          <button
            type="button"
            onClick={onClose}
            className="inline-flex items-center gap-1 text-xs text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2] whitespace-nowrap"
          >
            <X className="w-3.5 h-3.5" />
            <span>Close</span>
          </button>
        </div>
      </div>

      {previewLoading ? (
        <div className="text-xs text-[#57534E] dark:text-[#9CA3AF]">
          Loading file comparison...
        </div>
      ) : previewData ? (
        previewData.isBinary ? (
          <div className="text-xs text-[#57534E] dark:text-[#9CA3AF] font-mono tabular-nums">
            Binary file · Local:{' '}
            {previewData.existsLocal ? `${previewData.localSize} B` : 'missing'} ·
            Remote main:{' '}
            {previewData.existsRemote ? `${previewData.remoteSize} B` : 'missing'}
          </div>
        ) : (
          <div className="space-y-2">
            <div className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF]">
              {previewData.hasBase
                ? !previewData.existsRemote && previewData.existsLocal
                  ? `Deleted on GitHub main since baseline ${previewData.baseSha?.slice(0, 8)}`
                  : previewData.remoteUpdatedSinceBase
                    ? `Synced baseline: ${previewData.baseSha?.slice(0, 8)} · GitHub main updated (${previewData.remoteSha?.slice(0, 8)})`
                    : `Synced baseline: ${previewData.baseSha?.slice(0, 8)} (remote unchanged since baseline)`
                : 'No prior synced baseline recorded yet'}
            </div>

            {viewMode === 'line' ? (
              <div className="max-h-80 overflow-auto rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] font-mono text-[11px] leading-5">
                {visibleHunks.length === 0 ? (
                  <div className="p-4 text-[#57534E] dark:text-[#9CA3AF]">
                    No line differences detected.
                  </div>
                ) : (
                  <div className="divide-y divide-[#E5E0D5]/60 dark:divide-[#262B36]/60">
                    {visibleHunks.map((h, idx) => {
                      const rowClass =
                        h.type === 'add'
                          ? 'bg-emerald-50/85 dark:bg-emerald-950/35 text-emerald-900 dark:text-emerald-200'
                          : h.type === 'del'
                            ? 'bg-red-50/85 dark:bg-red-950/35 text-red-900 dark:text-red-200'
                            : 'text-[#57534E] dark:text-[#9CA3AF]';
                      const marker = h.type === 'add' ? '+' : h.type === 'del' ? '-' : ' ';
                      return (
                        <div
                          key={idx}
                          className={`flex items-start px-2.5 py-0.5 ${rowClass}`}
                        >
                          <span className="w-9 shrink-0 text-right pr-2 select-none opacity-65 tabular-nums">
                            {h.oldLine ?? ''}
                          </span>
                          <span className="w-9 shrink-0 text-right pr-2 select-none opacity-65 tabular-nums">
                            {h.newLine ?? ''}
                          </span>
                          <span className="w-4 shrink-0 select-none font-semibold">
                            {marker}
                          </span>
                          <span className="whitespace-pre-wrap break-all flex-1">
                            {h.text}
                          </span>
                        </div>
                      );
                    })}
                  </div>
                )}
                {(previewData.diffTruncated || previewData.truncated) && (
                  <div className="px-3 py-2 text-[11px] text-[#57534E] dark:text-[#9CA3AF] border-t border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317]">
                    Preview limited to the first 600 diff lines / 160 KB.
                  </div>
                )}
              </div>
            ) : (
              <div className="grid gap-3 md:grid-cols-2">
                <div>
                  <div className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF] mb-1">
                    Remote main (
                    {previewData.existsRemote ? `${previewData.remoteSize} B` : 'not on main'})
                  </div>
                  <pre className="max-h-72 overflow-auto rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] p-2.5 text-[11px] font-mono whitespace-pre-wrap break-all">
                    {previewData.remoteContent ?? '(absent on main)'}
                  </pre>
                </div>
                <div>
                  <div className="text-[11px] font-mono text-[#57534E] dark:text-[#9CA3AF] mb-1">
                    Local Container (
                    {previewData.existsLocal ? `${previewData.localSize} B` : 'not local'})
                  </div>
                  <pre className="max-h-72 overflow-auto rounded border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] p-2.5 text-[11px] font-mono whitespace-pre-wrap break-all">
                    {previewData.localContent ?? '(absent locally)'}
                  </pre>
                </div>
              </div>
            )}
          </div>
        )
      ) : null}
    </div>
  );
}
