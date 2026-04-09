import type { ReactNode } from "react";

import { MethodBadge } from "@/components/method-badge";

export function Endpoint({
  method,
  path,
  children,
}: {
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  path: string;
  children?: ReactNode;
}) {
  return (
    <div className="my-6 overflow-hidden rounded-sm border border-[var(--border)] bg-[var(--surface)] shadow-sm">
      <div className="flex flex-wrap items-center gap-3 border-b border-[var(--border)] px-4 py-3">
        <MethodBadge method={method} />
        <code className="font-mono text-sm font-medium text-[var(--foreground)]">
          {path}
        </code>
      </div>
      {children ? (
        <div className="px-4 py-4 text-sm leading-7 text-[var(--muted-foreground)]">
          {children}
        </div>
      ) : null}
    </div>
  );
}
