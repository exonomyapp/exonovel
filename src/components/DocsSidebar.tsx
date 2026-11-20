import { useEffect, useMemo, useRef, useState } from 'react';
import { ChevronDown, ChevronRight, FileText, Folder, FolderOpen, Search, X } from 'lucide-react';
import type { DocNode } from '../types/docs';

const DEFAULT_WIDTH = 276;
const MIN_WIDTH = 210;
const WIDTH_STORAGE_KEY = 'exonovel-docs-sidebar-width';

function clampWidth(value: number): number {
  return Math.max(MIN_WIDTH, Math.round(value));
}

interface DocsSidebarProps {
  tree: DocNode[];
  activePath: string;
  onSelectDoc: (path: string) => void;
  collapsed: boolean;
  mobileOpen: boolean;
  onCloseMobile: () => void;
}

function matchesQuery(node: DocNode, query: string): boolean {
  if (!query) return true;
  const q = query.toLowerCase();
  if (node.title.toLowerCase().includes(q) || node.name.toLowerCase().includes(q)) {
    return true;
  }
  if (node.headings?.some((h) => h.text.toLowerCase().includes(q))) {
    return true;
  }
  if (node.children?.some((child) => matchesQuery(child, q))) {
    return true;
  }
  return false;
}

function countFiles(nodes: DocNode[]): number {
  let count = 0;
  for (const node of nodes) {
    if (node.type === 'file') count += 1;
    if (node.children) count += countFiles(node.children);
  }
  return count;
}

export function DocsSidebar({
  tree,
  activePath,
  onSelectDoc,
  collapsed,
  mobileOpen,
  onCloseMobile,
}: DocsSidebarProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [sidebarWidth, setSidebarWidth] = useState<number>(() => {
    try {
      const saved = window.localStorage.getItem(WIDTH_STORAGE_KEY);
      const parsed = saved ? Number(saved) : NaN;
      return Number.isFinite(parsed) ? clampWidth(parsed) : DEFAULT_WIDTH;
    } catch {
      return DEFAULT_WIDTH;
    }
  });
  const [isResizing, setIsResizing] = useState(false);
  const asideRef = useRef<HTMLElement | null>(null);

  const [collapsedFolders, setCollapsedFolders] = useState<Record<string, boolean>>(() => {
    try {
      const saved = window.localStorage.getItem('exonovel-docs-folders-collapsed');
      return saved ? (JSON.parse(saved) as Record<string, boolean>) : {};
    } catch {
      return {};
    }
  });

  useEffect(() => {
    if (asideRef.current) {
      asideRef.current.style.setProperty('--sidebar-w', `${sidebarWidth}px`);
    }
    try {
      window.localStorage.setItem(WIDTH_STORAGE_KEY, String(sidebarWidth));
    } catch {
      // Ignore storage failures
    }
  }, [sidebarWidth, collapsed]);

  const startResize = (e: React.PointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = sidebarWidth;
    setIsResizing(true);
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'col-resize';

    const handleMove = (moveEvent: PointerEvent) => {
      setSidebarWidth(clampWidth(startWidth + (moveEvent.clientX - startX)));
    };

    const handleEnd = () => {
      setIsResizing(false);
      document.body.style.userSelect = '';
      document.body.style.cursor = '';
      window.removeEventListener('pointermove', handleMove);
      window.removeEventListener('pointerup', handleEnd);
      window.removeEventListener('pointercancel', handleEnd);
    };

    window.addEventListener('pointermove', handleMove);
    window.addEventListener('pointerup', handleEnd);
    window.addEventListener('pointercancel', handleEnd);
  };

  const handleResizeKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    const step = e.shiftKey ? 32 : 16;
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      setSidebarWidth((prev) => clampWidth(prev + step));
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      setSidebarWidth((prev) => clampWidth(prev - step));
    } else if (e.key === 'Home') {
      e.preventDefault();
      setSidebarWidth(MIN_WIDTH);
    }
  };

  const totalFiles = useMemo(() => countFiles(tree), [tree]);

  const filteredTree = useMemo(() => {
    const trimmed = searchQuery.trim();
    if (!trimmed) return tree;

    const filterNodes = (nodes: DocNode[]): DocNode[] =>
      nodes
        .filter((node) => matchesQuery(node, trimmed))
        .map((node) =>
          node.type === 'folder' && node.children
            ? { ...node, children: filterNodes(node.children) }
            : node
        );

    return filterNodes(tree);
  }, [tree, searchQuery]);

  const toggleFolder = (folderPath: string) => {
    setCollapsedFolders((prev) => {
      const next = {
        ...prev,
        [folderPath]: !prev[folderPath],
      };
      try {
        window.localStorage.setItem('exonovel-docs-folders-collapsed', JSON.stringify(next));
      } catch {
        // Ignore storage failures
      }
      return next;
    });
  };

  const renderNode = (node: DocNode, depth = 0) => {
    if (node.type === 'folder') {
      const isSearching = searchQuery.trim().length > 0;
      const isCollapsed = !isSearching && Boolean(collapsedFolders[node.path]);
      const folderFileCount = countFiles(node.children ?? []);

      return (
        <div key={node.id} className="mb-1">
          <button
            type="button"
            onClick={() => toggleFolder(node.path)}
            style={{ paddingLeft: `${depth * 14 + 10}px` }}
            className="w-full flex items-center justify-between gap-2 py-2 pr-3 text-left text-xs font-semibold tracking-wide text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2] rounded-md hover:bg-[#EAE5DB]/70 dark:hover:bg-[#1E232D] transition-colors min-h-[40px] focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-[#1D4ED8]"
            aria-expanded={!isCollapsed}
          >
            <span className="flex items-center gap-2 min-w-0">
              {isCollapsed ? (
                <ChevronRight className="w-3.5 h-3.5 shrink-0 opacity-70" />
              ) : (
                <ChevronDown className="w-3.5 h-3.5 shrink-0 opacity-70" />
              )}
              {isCollapsed ? (
                <Folder className="w-3.5 h-3.5 shrink-0 opacity-75" />
              ) : (
                <FolderOpen className="w-3.5 h-3.5 shrink-0 opacity-75" />
              )}
              <span className="truncate">{node.title}</span>
            </span>
            <span className="text-[11px] font-normal tabular-nums opacity-70 shrink-0">
              {folderFileCount}
            </span>
          </button>

          {!isCollapsed && node.children && (
            <div className="mt-0.5 space-y-0.5">
              {node.children.map((child) => renderNode(child, depth + 1))}
            </div>
          )}
        </div>
      );
    }

    const isActive = node.path === activePath;

    return (
      <button
        key={node.id}
        type="button"
        onClick={() => {
          onSelectDoc(node.path);
          onCloseMobile();
        }}
        style={{ paddingLeft: `${depth * 14 + 12}px` }}
        className={`w-full flex items-center justify-between gap-2 py-2 pr-3 text-left text-[13.5px] rounded-md transition-colors min-h-[40px] focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-[#1D4ED8] ${
          isActive
            ? 'bg-[#FFFFFF] dark:bg-[#1E2430] text-[#1D4ED8] dark:text-[#60A5FA] font-semibold border border-[#E5E0D5] dark:border-[#2C3342]'
            : 'text-[#292524] dark:text-[#D1D5DB] hover:bg-[#EAE5DB]/70 dark:hover:bg-[#1E232D] border border-transparent'
        }`}
        aria-current={isActive ? 'page' : undefined}
      >
        <span className="flex items-center gap-2 min-w-0">
          <FileText
            className={`w-3.5 h-3.5 shrink-0 ${
              isActive ? 'text-[#1D4ED8] dark:text-[#60A5FA]' : 'opacity-60'
            }`}
          />
          <span className="truncate">{node.title}</span>
        </span>
      </button>
    );
  };

  const sidebarContent = (
    <div className="flex flex-col h-full bg-[#F3F0EA] dark:bg-[#171A21] border-r border-[#E5E0D5] dark:border-[#262B36]">
      <div className="px-5 pt-5 pb-4 border-b border-[#E5E0D5] dark:border-[#262B36] flex items-center justify-between gap-2">
        <div className="min-w-0">
          <div className="font-display text-lg font-semibold tracking-tight text-[#1C1917] dark:text-[#ECEEF2] whitespace-nowrap">
            ExoNovel
          </div>
          <div className="text-xs text-[#57534E] dark:text-[#9CA3AF] mt-0.5 tabular-nums">
            Documentation · {totalFiles} {totalFiles === 1 ? 'file' : 'files'}
          </div>
        </div>
        <button
          type="button"
          onClick={onCloseMobile}
          className="lg:hidden p-2 rounded-md text-[#57534E] dark:text-[#9CA3AF] hover:bg-[#EAE5DB] dark:hover:bg-[#232834] min-h-[40px] min-w-[40px] flex items-center justify-center"
          aria-label="Close navigation"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="px-4 py-3 border-b border-[#E5E0D5] dark:border-[#262B36]">
        <div className="relative">
          <Search className="w-3.5 h-3.5 text-[#78716C] dark:text-[#9CA3AF] absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
          <input
            type="search"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Filter documents..."
            aria-label="Filter documents"
            className="w-full pl-8 pr-8 py-2 text-xs bg-[#FAF8F4] dark:bg-[#111317] text-[#1C1917] dark:text-[#ECEEF2] placeholder:text-[#78716C] dark:placeholder:text-[#6B7280] border border-[#E5E0D5] dark:border-[#262B36] rounded-md focus:outline-none focus:border-[#1D4ED8] dark:focus:border-[#60A5FA]"
          />
          {searchQuery && (
            <button
              type="button"
              onClick={() => setSearchQuery('')}
              className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-[#78716C] hover:text-[#1C1917] dark:hover:text-[#ECEEF2]"
              aria-label="Clear filter"
            >
              <X className="w-3 h-3" />
            </button>
          )}
        </div>
      </div>

      <nav className="flex-1 overflow-y-auto px-3 py-3" aria-label="Documentation Explorer">
        {filteredTree.length > 0 ? (
          filteredTree.map((node) => renderNode(node))
        ) : (
          <div className="px-3 py-6 text-xs text-[#78716C] dark:text-[#9CA3AF] text-center">
            No matching documents found.
          </div>
        )}
      </nav>
    </div>
  );

  return (
    <>
      {!collapsed && (
        <aside
          ref={asideRef}
          className="hidden lg:block relative w-[var(--sidebar-w,276px)] shrink-0 h-screen sticky top-0"
        >
          {sidebarContent}
          <div
            role="separator"
            tabIndex={0}
            aria-orientation="vertical"
            aria-label="Resize documentation sidebar"
            aria-valuemin={MIN_WIDTH}
            aria-valuenow={sidebarWidth}
            onPointerDown={startResize}
            onDoubleClick={() => setSidebarWidth(DEFAULT_WIDTH)}
            onKeyDown={handleResizeKeyDown}
            title="Drag to resize sidebar (double-click to reset)"
            className={`absolute top-0 right-0 w-1.5 h-full cursor-col-resize transition-colors focus-visible:outline-none focus-visible:bg-[#1D4ED8] ${
              isResizing
                ? 'bg-[#1D4ED8]/60 dark:bg-[#60A5FA]/60'
                : 'hover:bg-[#1D4ED8]/30 dark:hover:bg-[#60A5FA]/30'
            }`}
          />
        </aside>
      )}

      {mobileOpen && (
        <div className="fixed inset-0 z-50 lg:hidden flex">
          <div
            className="fixed inset-0 bg-black/40"
            onClick={onCloseMobile}
            aria-hidden="true"
          />
          <aside className="relative w-[290px] max-w-[85vw] h-full z-10">
            {sidebarContent}
          </aside>
        </div>
      )}
    </>
  );
}
