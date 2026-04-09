const styles = {
  GET: "bg-emerald-500/12 text-emerald-700 ring-1 ring-emerald-500/20 dark:text-emerald-300",
  POST: "bg-blue-500/12 text-blue-700 ring-1 ring-blue-500/20 dark:text-blue-300",
  PUT: "bg-amber-500/12 text-amber-700 ring-1 ring-amber-500/20 dark:text-amber-300",
  PATCH: "bg-fuchsia-500/12 text-fuchsia-700 ring-1 ring-fuchsia-500/20 dark:text-fuchsia-300",
  DELETE:
    "bg-rose-500/12 text-rose-700 ring-1 ring-rose-500/20 dark:text-rose-300",
} as const;

type Method = keyof typeof styles;

export function MethodBadge({ method }: { method: Method }) {
  return (
    <span
      className={`inline-flex items-center rounded-sm px-2.5 py-1 font-mono text-[10px] font-semibold uppercase tracking-[0.12em] ${styles[method]}`}
    >
      {method}
    </span>
  );
}
