import { useState, type ReactNode } from 'react';
import ReactMarkdown, { type Components } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeSlug from 'rehype-slug';
import { ArrowLeft, ArrowRight, Check, Copy, Link2 } from 'lucide-react';
import type { DocNode, DocPayload } from '../types/docs';

interface MarkdownReaderProps {
  doc: DocPayload;
  prevDoc: DocNode | null;
  nextDoc: DocNode | null;
  onNavigateDoc: (path: string) => void;
}

function resolveRelativeDocPath(currentDocPath: string, href: string): string | null {
  if (!href || href.startsWith('http://') || href.startsWith('https://') || href.startsWith('mailto:')) {
    return null;
  }
  const cleanHref = href.split('#')[0];
  if (!cleanHref.toLowerCase().endsWith('.md')) return null;

  const baseParts = currentDocPath.split('/').slice(0, -1);
  const targetParts = cleanHref.replace(/^\//, '').split('/');

  const stack = cleanHref.startsWith('/') ? [] : [...baseParts];
  for (const part of targetParts) {
    if (!part || part === '.') continue;
    if (part === '..') {
      stack.pop();
    } else {
      stack.push(part);
    }
  }
  return stack.join('/');
}

function CodeBlock({ className, children }: { className?: string; children: ReactNode }) {
  const [copied, setCopied] = useState(false);
  const rawText = String(children ?? '').replace(/\n$/, '');
  const langMatch = /language-([^\s]+)/.exec(className || '');
  const language = langMatch ? langMatch[1] : '';

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(rawText);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      // Clipboard write unavailable in restricted frames
    }
  };

  return (
    <div className="my-6 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/80 dark:bg-[#171A21] overflow-hidden">
      <div className="flex items-center justify-between px-4 py-2 border-b border-[#E5E0D5] dark:border-[#262B36] text-xs text-[#57534E] dark:text-[#9CA3AF]">
        <span className="font-mono">{language || 'text'}</span>
        <button
          type="button"
          onClick={handleCopy}
          className="inline-flex items-center gap-1.5 px-2 py-1 rounded hover:bg-[#EAE5DB] dark:hover:bg-[#232834] text-[#44403C] dark:text-[#D1D5DB] transition-colors whitespace-nowrap shrink-0"
          aria-label="Copy code"
        >
          {copied ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
          <span>{copied ? 'Copied' : 'Copy'}</span>
        </button>
      </div>
      <pre className="p-4 overflow-x-auto text-[13.5px] leading-relaxed font-mono text-[#1C1917] dark:text-[#ECEEF2]">
        <code>{rawText}</code>
      </pre>
    </div>
  );
}

export function MarkdownReader({
  doc,
  prevDoc,
  nextDoc,
  onNavigateDoc,
}: MarkdownReaderProps) {
  const formattedUpdated = new Date(doc.updatedAt).toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });

  const components: Components = {
    h1({ id, children, ...props }) {
      return (
        <h1
          id={id}
          {...props}
          className="font-display text-3xl sm:text-4xl font-semibold tracking-tight text-[#1C1917] dark:text-[#ECEEF2] mt-2 mb-6 leading-tight text-balance scroll-mt-20"
        >
          {children}
        </h1>
      );
    },
    h2({ id, children, ...props }) {
      return (
        <h2
          id={id}
          {...props}
          className="group font-display text-2xl font-semibold tracking-tight text-[#1C1917] dark:text-[#ECEEF2] mt-11 mb-4 pb-2 border-b border-[#E5E0D5] dark:border-[#262B36] leading-snug scroll-mt-20 flex items-center gap-2"
        >
          <span>{children}</span>
          {id && (
            <a
              href={`#${id}`}
              aria-label="Link to section"
              className="opacity-0 group-hover:opacity-70 hover:opacity-100 text-[#57534E] dark:text-[#9CA3AF] transition-opacity"
            >
              <Link2 className="w-4 h-4" />
            </a>
          )}
        </h2>
      );
    },
    h3({ id, children, ...props }) {
      return (
        <h3
          id={id}
          {...props}
          className="group font-display text-xl font-medium text-[#1C1917] dark:text-[#ECEEF2] mt-8 mb-3 leading-snug scroll-mt-20 flex items-center gap-2"
        >
          <span>{children}</span>
          {id && (
            <a
              href={`#${id}`}
              aria-label="Link to subsection"
              className="opacity-0 group-hover:opacity-70 hover:opacity-100 text-[#57534E] dark:text-[#9CA3AF] transition-opacity"
            >
              <Link2 className="w-3.5 h-3.5" />
            </a>
          )}
        </h3>
      );
    },
    h4({ id, children, ...props }) {
      return (
        <h4
          id={id}
          {...props}
          className="text-base font-semibold text-[#1C1917] dark:text-[#ECEEF2] mt-6 mb-2 scroll-mt-20"
        >
          {children}
        </h4>
      );
    },
    p({ children, ...props }) {
      return (
        <p
          {...props}
          className="text-[15.5px] sm:text-base leading-[1.75] text-[#292524] dark:text-[#D1D5DB] my-4"
        >
          {children}
        </p>
      );
    },
    ul({ children, className, ...props }) {
      const isTaskList = className?.includes('contains-task-list');
      return (
        <ul
          {...props}
          className={`my-4 space-y-2 text-[15.5px] sm:text-base leading-[1.7] text-[#292524] dark:text-[#D1D5DB] ${
            isTaskList ? 'list-none pl-1' : 'list-disc pl-6'
          }`}
        >
          {children}
        </ul>
      );
    },
    ol({ children, ...props }) {
      return (
        <ol
          {...props}
          className="my-4 list-decimal pl-6 space-y-2 text-[15.5px] sm:text-base leading-[1.7] text-[#292524] dark:text-[#D1D5DB]"
        >
          {children}
        </ol>
      );
    },
    li({ children, className, ...props }) {
      const isTaskItem = className?.includes('task-list-item');
      return (
        <li
          {...props}
          className={isTaskItem ? 'flex items-start gap-2.5' : 'pl-1'}
        >
          {children}
        </li>
      );
    },
    input({ type, checked, ...props }) {
      if (type === 'checkbox') {
        return (
          <input
            type="checkbox"
            checked={Boolean(checked)}
            readOnly
            className="mt-1.5 h-4 w-4 rounded border-[#D6D0C4] dark:border-[#374151] accent-[#1D4ED8]"
            {...props}
          />
        );
      }
      return <input type={type} {...props} />;
    },
    blockquote({ children, ...props }) {
      return (
        <blockquote
          {...props}
          className="my-6 px-5 py-3.5 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/65 dark:bg-[#171A21] text-[#44403C] dark:text-[#CBD5E1] [&>p]:my-1"
        >
          {children}
        </blockquote>
      );
    },
    hr() {
      return <hr className="my-8 border-0 border-t border-[#E5E0D5] dark:border-[#262B36]" />;
    },
    a({ href, children, ...props }) {
      const internalTarget = href ? resolveRelativeDocPath(doc.path, href) : null;
      if (internalTarget) {
        return (
          <a
            href={`?doc=${encodeURIComponent(internalTarget)}`}
            onClick={(e) => {
              e.preventDefault();
              onNavigateDoc(internalTarget);
            }}
            className="text-[#1D4ED8] dark:text-[#60A5FA] underline decoration-[#1D4ED8]/40 underline-offset-2 hover:decoration-[#1D4ED8]"
            {...props}
          >
            {children}
          </a>
        );
      }

      const isAnchor = href?.startsWith('#');
      return (
        <a
          href={href}
          target={isAnchor ? undefined : '_blank'}
          rel={isAnchor ? undefined : 'noreferrer'}
          className="text-[#1D4ED8] dark:text-[#60A5FA] underline decoration-[#1D4ED8]/40 underline-offset-2 hover:decoration-[#1D4ED8]"
          {...props}
        >
          {children}
        </a>
      );
    },
    pre({ children }) {
      return <>{children}</>;
    },
    code({ className, children, ...props }) {
      const text = String(children ?? '');
      const isBlock = Boolean(className) || text.includes('\n');
      if (isBlock) {
        return <CodeBlock className={className}>{children}</CodeBlock>;
      }
      return (
        <code
          {...props}
          className="font-mono text-[0.875em] px-1.5 py-0.5 rounded bg-[#EFECE5] dark:bg-[#1F2430] text-[#1C1917] dark:text-[#E5E7EB]"
        >
          {children}
        </code>
      );
    },
    table({ children, ...props }) {
      return (
        <div className="my-6 overflow-x-auto rounded-lg border border-[#E5E0D5] dark:border-[#262B36]">
          <table
            {...props}
            className="w-full text-left border-collapse text-sm tabular-nums"
          >
            {children}
          </table>
        </div>
      );
    },
    thead({ children, ...props }) {
      return (
        <thead
          {...props}
          className="bg-[#F3F0EA] dark:bg-[#171A21] border-b border-[#E5E0D5] dark:border-[#262B36]"
        >
          {children}
        </thead>
      );
    },
    th({ children, ...props }) {
      return (
        <th
          {...props}
          className="px-4 py-2.5 font-semibold text-[#1C1917] dark:text-[#ECEEF2]"
        >
          {children}
        </th>
      );
    },
    td({ children, ...props }) {
      return (
        <td
          {...props}
          className="px-4 py-2.5 border-t border-[#E5E0D5] dark:border-[#262B36] text-[#292524] dark:text-[#D1D5DB] align-top"
        >
          {children}
        </td>
      );
    },
  };

  return (
    <article className="flex-1 min-w-0 w-full">
      <div className="mb-4 flex flex-wrap items-center gap-x-2.5 gap-y-1 text-xs text-[#57534E] dark:text-[#9CA3AF] tabular-nums">
        <span className="font-mono">{doc.path}</span>
        <span aria-hidden="true">·</span>
        <span>{doc.wordCount.toLocaleString()} words</span>
        <span aria-hidden="true">·</span>
        <span>{doc.readingMinutes} min read</span>
        <span aria-hidden="true">·</span>
        <span>Updated {formattedUpdated}</span>
      </div>

      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeSlug]}
        components={components}
      >
        {doc.content}
      </ReactMarkdown>

      {(prevDoc || nextDoc) && (
        <footer className="mt-14 pt-6 border-t border-[#E5E0D5] dark:border-[#262B36] grid grid-cols-1 sm:grid-cols-2 gap-4">
          {prevDoc ? (
            <button
              type="button"
              onClick={() => onNavigateDoc(prevDoc.path)}
              className="flex flex-col items-start gap-1 p-4 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#F3F0EA]/70 dark:hover:bg-[#171A21] text-left transition-colors min-h-[44px]"
            >
              <span className="inline-flex items-center gap-1.5 text-xs text-[#57534E] dark:text-[#9CA3AF]">
                <ArrowLeft className="w-3.5 h-3.5" />
                <span>Previous</span>
              </span>
              <span className="text-sm font-semibold text-[#1C1917] dark:text-[#ECEEF2] truncate w-full">
                {prevDoc.title}
              </span>
            </button>
          ) : (
            <div />
          )}

          {nextDoc ? (
            <button
              type="button"
              onClick={() => onNavigateDoc(nextDoc.path)}
              className="flex flex-col items-end gap-1 p-4 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] hover:bg-[#F3F0EA]/70 dark:hover:bg-[#171A21] text-right transition-colors min-h-[44px]"
            >
              <span className="inline-flex items-center gap-1.5 text-xs text-[#57534E] dark:text-[#9CA3AF]">
                <span>Next</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </span>
              <span className="text-sm font-semibold text-[#1C1917] dark:text-[#ECEEF2] truncate w-full">
                {nextDoc.title}
              </span>
            </button>
          ) : (
            <div />
          )}
        </footer>
      )}
    </article>
  );
}
