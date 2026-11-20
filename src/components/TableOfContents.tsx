import type { DocHeading } from '../types/docs';

interface TableOfContentsProps {
  headings: DocHeading[];
  activeId: string;
}

export function TableOfContents({ headings, activeId }: TableOfContentsProps) {
  if (headings.length === 0) return null;

  return (
    <aside className="hidden xl:block w-[220px] shrink-0">
      <div className="sticky top-20 pl-6 border-l border-[#E5E0D5] dark:border-[#262B36]">
        <div className="text-xs font-semibold text-[#57534E] dark:text-[#9CA3AF] mb-3">
          On this page
        </div>
        <ul className="space-y-1.5 text-[12.5px] leading-snug">
          {headings.map((heading) => {
            const isActive = activeId === heading.id;
            return (
              <li
                key={heading.id}
                className={heading.level === 3 ? 'pl-3' : undefined}
              >
                <a
                  href={`#${heading.id}`}
                  onClick={(e) => {
                    e.preventDefault();
                    const el = document.getElementById(heading.id);
                    if (el) {
                      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
                      window.history.replaceState(null, '', `#${heading.id}`);
                    }
                  }}
                  className={`block py-1 transition-colors ${
                    isActive
                      ? 'text-[#1D4ED8] dark:text-[#60A5FA] font-medium'
                      : 'text-[#57534E] dark:text-[#9CA3AF] hover:text-[#1C1917] dark:hover:text-[#ECEEF2]'
                  }`}
                >
                  {heading.text}
                </a>
              </li>
            );
          })}
        </ul>
      </div>
    </aside>
  );
}
