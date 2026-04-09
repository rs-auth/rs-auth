import type { ReactNode } from "react";

export function CodeFrame({
  title,
  children,
}: {
  title?: string;
  children: ReactNode;
}) {
  return (
    <div className="my-6 overflow-hidden rounded-sm border border-white/[0.06] bg-[#050505] shadow-lg shadow-black/10">
      {title ? (
        <div className="border-b border-white/[0.06] px-4 py-2 font-mono text-xs font-medium uppercase tracking-[0.14em] text-[#8ea4d9]">
          {title}
        </div>
      ) : null}
      <div className="px-4 py-4 text-sm text-[#dbe6ff]">{children}</div>
    </div>
  );
}
