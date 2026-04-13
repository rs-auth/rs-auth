import type { ReactNode } from "react";

const toneStyles = {
  info: {
    wrapper:
      "border-sky-500/20 bg-sky-500/8 dark:text-sky-100 text-sky-500 dark:border-sky-400/20 dark:bg-sky-400/8",
    label: "text-sky-700 dark:text-sky-300",
  },
  warn: {
    wrapper:
      "border-amber-500/20 bg-amber-500/8 text-amber-950 dark:border-amber-400/20 dark:bg-amber-400/10 dark:text-amber-100",
    label: "text-amber-700 dark:text-amber-300",
  },
  success: {
    wrapper:
      "border-emerald-500/20 bg-emerald-500/8 text-emerald-950 dark:border-emerald-400/20 dark:bg-emerald-400/10 dark:text-emerald-100",
    label: "text-emerald-700 dark:text-emerald-300",
  },
} as const;

type Tone = keyof typeof toneStyles;

export function Callout({
  title,
  tone = "info",
  children,
}: {
  title: string;
  tone?: Tone;
  children: ReactNode;
}) {
  const style = toneStyles[tone];

  return (
    <div
      className={`my-6 rounded-sm border px-4 py-4 backdrop-blur ${style.wrapper}`}
    >
      <div
        className={`mb-2 font-mono text-xs font-semibold uppercase tracking-[0.14em] ${style.label}`}
      >
        {title}
      </div>
      <div className="text-sm leading-7 text-current/90">{children}</div>
    </div>
  );
}
