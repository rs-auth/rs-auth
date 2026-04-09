"use client";

import { Search } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { Logo } from "./logo";
import { ThemeToggle } from "./theme-toggle";

interface NavItem {
  name: string;
  href: string;
  external?: boolean;
}

const navItems: NavItem[] = [
  { name: "docs", href: "/docs" },
  {
    name: "github",
    href: "https://github.com/rs-auth/rs-auth",
    external: true,
  },
];

export function Nav() {
  const pathname = usePathname() || "/";
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const isDocs = pathname.startsWith("/docs");
  const isHome = pathname === "/";

  useEffect(() => {
    document.body.style.overflow = mobileMenuOpen ? "hidden" : "";
    return () => {
      document.body.style.overflow = "";
    };
  }, [mobileMenuOpen]);

  useEffect(() => {
    const mql = window.matchMedia("(min-width: 1024px)");
    const handler = () => {
      if (mql.matches) setMobileMenuOpen(false);
    };
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  }, []);

  const isActive = (href: string) =>
    pathname === href || (href === "/docs" && isDocs);

  const leftPaneWidthClass = isDocs ? "w-[22vw] max-w-[300px]" : "w-[40%]";

  const tabDividerClass = isDocs
    ? "border-foreground/4"
    : "border-foreground/[0.06]";
  const activeTabBorderClass = isDocs
    ? "border-b-foreground/50"
    : "border-b-foreground/60";

  return (
    <>
      <div className="fixed top-0 left-0 right-0 z-[99] flex items-start pointer-events-none">
        {/* Left pane — Logo (desktop) */}
        <div
          className={cn(
            leftPaneWidthClass,
            "border-foreground/[0.06] border-r nav-desktop flex h-[var(--landing-topbar-height)] items-stretch shrink-0 pointer-events-auto transition-[width] duration-300 ease-out",
          )}
        >
          <Link
            href="/"
            className="flex h-full items-center gap-1 px-4 transition-colors duration-150"
          >
            <Logo className="h-4 w-auto" />
          </Link>
        </div>

        {/* Mobile — Logo + controls */}
        <div className="nav-mobile flex items-center justify-between w-full h-[var(--landing-topbar-height)] pointer-events-auto bg-background border-b border-foreground/[0.06]">
          <Link
            href="/"
            className="flex h-full items-center gap-1 px-4 transition-colors duration-150"
          >
            <Logo className="h-3.5 w-auto" />
          </Link>
          <div className="flex items-center gap-1 pr-2">
            {isDocs && (
              <button
                type="button"
                onClick={() => {
                  window.dispatchEvent(
                    new KeyboardEvent("keydown", {
                      key: "k",
                      metaKey: true,
                      bubbles: true,
                    }),
                  );
                }}
                className="flex items-center justify-center size-8 text-foreground/50 hover:text-foreground/80 transition-colors"
                aria-label="Search"
              >
                <Search className="size-4" />
              </button>
            )}
            <div className="flex items-center justify-center size-8 text-foreground/50 [&_button]:text-foreground/50 [&_button:hover]:text-foreground/80">
              <ThemeToggle />
            </div>
            <button
              type="button"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              className="flex items-center justify-center size-8 text-foreground/75 dark:text-foreground/60 hover:text-foreground/85 transition-colors"
              aria-label="Toggle menu"
            >
              {mobileMenuOpen ? (
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                >
                  <title>Close</title>
                  <path
                    fill="currentColor"
                    d="M19 6.41L17.59 5L12 10.59L6.41 5L5 6.41L10.59 12L5 17.59L6.41 19L12 13.41L17.59 19L19 17.59L13.41 12z"
                  />
                </svg>
              ) : (
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                >
                  <title>Menu</title>
                  <path
                    fill="currentColor"
                    d="M3 18h18v-2H3zm0-5h18v-2H3zm0-7v2h18V6z"
                  />
                </svg>
              )}
            </button>
          </div>
        </div>

        {/* Right pane — Tab links (desktop) */}
        <div
          className={cn(
            "nav-desktop flex-1 flex h-[calc(var(--landing-topbar-height)+1px)] items-stretch border-b bg-background pointer-events-auto min-w-0",
            isDocs ? "border-foreground/5" : "",
          )}
        >
          {/* Home tab */}
          <div className="flex-1">
            <Link
              href="/"
              className={cn(
                "group/tab relative flex items-center justify-center gap-1.5 px-2 xl:px-4 py-3 h-full border-r transition-colors duration-150",
                tabDividerClass,
                isHome
                  ? `bg-background border-b-2 ${activeTabBorderClass}`
                  : "bg-transparent hover:bg-foreground/[0.03]",
              )}
            >
              <span
                className={cn(
                  "font-mono text-xs uppercase tracking-wider transition-colors duration-150 whitespace-nowrap",
                  isHome
                    ? "text-foreground"
                    : "text-foreground/65 dark:text-foreground/50 group-hover/tab:text-foreground/75",
                )}
              >
                readme
              </span>
            </Link>
          </div>

          {/* Nav tabs */}
          {navItems.map((item) => {
            const active = isActive(item.href);
            return (
              <div key={item.name} className="flex-1">
                <Link
                  href={item.href}
                  target={item.external ? "_blank" : undefined}
                  rel={item.external ? "noreferrer" : undefined}
                  className={cn(
                    "group/tab relative flex items-center justify-center gap-1.5 px-2 xl:px-4 py-3 h-full border-r transition-colors duration-150",
                    tabDividerClass,
                    active
                      ? `bg-background border-b-2 ${activeTabBorderClass}`
                      : "bg-transparent hover:bg-foreground/[0.03]",
                  )}
                >
                  <span
                    className={cn(
                      "font-mono text-xs uppercase tracking-wider transition-colors duration-150 whitespace-nowrap",
                      active
                        ? "text-foreground"
                        : "text-foreground/65 dark:text-foreground/50 group-hover/tab:text-foreground/75",
                    )}
                  >
                    {item.name}
                  </span>
                  {item.external && (
                    <svg
                      className="h-2 w-2 opacity-40"
                      viewBox="0 0 10 10"
                      fill="none"
                    >
                      <title>External</title>
                      <path
                        d="M1 9L9 1M9 1H3M9 1V7"
                        stroke="currentColor"
                        strokeWidth="1.2"
                      />
                    </svg>
                  )}
                </Link>
              </div>
            );
          })}

          {/* Get Started CTA */}
          <div className="flex items-stretch shrink-0">
            <Link
              href="/docs"
              className="flex items-center cursor-pointer gap-1.5 px-5 py-3 bg-foreground text-background hover:opacity-90 transition-colors duration-150"
            >
              <span className="font-mono text-xs uppercase tracking-wider">
                get started
              </span>
            </Link>
          </div>
        </div>
      </div>

      {/* Mobile menu overlay */}
      {mobileMenuOpen && (
        <div className="nav-mobile fixed inset-0 z-[98] w-full bg-background/95 backdrop-blur-sm">
          <div className="flex h-full flex-col pt-[var(--landing-topbar-height)]">
            <div className="flex-1 min-h-0 overflow-y-auto">
              <Link
                href="/"
                onClick={() => setMobileMenuOpen(false)}
                className={cn(
                  "flex items-center gap-2.5 px-5 py-3.5 border-b border-foreground/6 transition-colors font-mono text-base uppercase tracking-wider",
                  isHome
                    ? "text-foreground bg-foreground/4"
                    : "text-foreground/75 dark:text-foreground/60 hover:bg-foreground/3",
                )}
              >
                readme
              </Link>

              {navItems.map((item) => (
                <Link
                  key={item.name}
                  href={item.href}
                  target={item.external ? "_blank" : undefined}
                  rel={item.external ? "noreferrer" : undefined}
                  onClick={() => setMobileMenuOpen(false)}
                  className={cn(
                    "flex items-center gap-2.5 px-5 py-3.5 border-b border-foreground/6 transition-colors font-mono text-base uppercase tracking-wider",
                    isActive(item.href)
                      ? "text-foreground bg-foreground/4"
                      : "text-foreground/75 dark:text-foreground/60 hover:bg-foreground/3",
                  )}
                >
                  {item.name}
                  {item.external && (
                    <svg
                      className="h-2.5 w-2.5 opacity-40"
                      viewBox="0 0 10 10"
                      fill="none"
                    >
                      <title>External</title>
                      <path
                        d="M1 9L9 1M9 1H3M9 1V7"
                        stroke="currentColor"
                        strokeWidth="1.2"
                      />
                    </svg>
                  )}
                </Link>
              ))}
            </div>

            <div className="shrink-0 border-t border-foreground/[0.06] bg-background px-5 py-4">
              <Link
                href="/docs"
                onClick={() => setMobileMenuOpen(false)}
                className="flex items-center justify-center gap-1.5 w-full py-3 bg-foreground text-background font-mono text-sm uppercase tracking-wider transition-opacity hover:opacity-90"
              >
                get started
              </Link>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
