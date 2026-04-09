"use client";

import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";

export interface TOCItem {
  title: string;
  url: string;
  depth: number;
}

function InlineCode({ children }: { children: string }) {
  // Render backtick-wrapped segments as <code>
  const parts = children.split(/(`[^`]+`)/g);
  return (
    <>
      {parts.map((part) => {
        if (part.startsWith("`") && part.endsWith("`")) {
          const inner = part.slice(1, -1);
          return (
            <code
              key={inner}
              className="rounded-sm border border-foreground/10 bg-foreground/[0.06] px-1 py-px font-mono text-[0.8em]"
            >
              {inner}
            </code>
          );
        }
        return <span key={part}>{part}</span>;
      })}
    </>
  );
}

export function TableOfContents({ items }: { items: TOCItem[] }) {
  const [active, setActive] = useState<string>("");
  const observerRef = useRef<IntersectionObserver | null>(null);

  useEffect(() => {
    const headings = items.map((item) => {
      const id = item.url.replace("#", "");
      return document.getElementById(id);
    });

    observerRef.current = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setActive(`#${entry.target.id}`);
            break;
          }
        }
      },
      { rootMargin: "0px 0px -60% 0px", threshold: 0 },
    );

    for (const el of headings) {
      if (el) observerRef.current.observe(el);
    }

    return () => observerRef.current?.disconnect();
  }, [items]);

  if (items.length === 0) return null;

  return (
    <div className="sticky top-[calc(var(--landing-topbar-height)+2rem)] w-56 shrink-0 select-none">
      <div className="flex items-center gap-2 mb-4 text-foreground/50">
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.75"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <line x1="3" y1="6" x2="21" y2="6" />
          <line x1="3" y1="12" x2="15" y2="12" />
          <line x1="3" y1="18" x2="12" y2="18" />
        </svg>
        <span className="text-xs font-mono uppercase tracking-wider">
          On this page
        </span>
      </div>

      <div className="relative border-l border-foreground/10 pl-px flex flex-col">
        {items.map((item) => {
          const isActive = active === item.url;
          const indent = (item.depth - 2) * 12;
          return (
            <a
              key={item.url}
              href={item.url}
              style={{ paddingLeft: `${indent + 12}px` }}
              className={cn(
                "relative py-1 pr-2 text-[13px] leading-5 transition-colors duration-150",
                "hover:text-foreground",
                isActive ? "text-foreground" : "text-foreground/50",
              )}
            >
              {isActive && (
                <span
                  aria-hidden="true"
                  className="absolute left-[-1px] top-0 h-full w-px bg-foreground/70"
                />
              )}
              <InlineCode>{item.title}</InlineCode>
            </a>
          );
        })}
      </div>
    </div>
  );
}
