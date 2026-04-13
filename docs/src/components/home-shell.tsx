import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface HomeShellProps {
  left: ReactNode;
  right: ReactNode;
  leftClassName?: string;
  rightClassName?: string;
}

export function HomeShell({
  left,
  right,
  leftClassName,
  rightClassName,
}: HomeShellProps) {
  return (
    <div className="home-page relative pt-[var(--landing-topbar-height)] lg:pt-0">
      <div className="relative text-foreground">
        <div className="home-shell flex flex-col lg:flex-row">
          <div className={cn("home-left", leftClassName)}>{left}</div>
          <div className={cn("home-right", rightClassName)}>{right}</div>
        </div>
      </div>
    </div>
  );
}
