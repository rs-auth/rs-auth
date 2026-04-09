"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { ThemeToggle } from "./theme-toggle";

interface NavItem {
  title: string;
  href: string;
}

interface NavSection {
  title: string;
  items: NavItem[];
}

const navSections: NavSection[] = [
  {
    title: "Get Started",
    items: [
      { title: "Introduction", href: "/docs" },
      { title: "Installation", href: "/docs/installation" },
      { title: "Quick Start", href: "/docs/quick-start" },
    ],
  },
  {
    title: "Concepts",
    items: [
      { title: "Sessions", href: "/docs/concepts/sessions" },
      {
        title: "Email Verification",
        href: "/docs/concepts/email-verification",
      },
    ],
  },
  {
    title: "Configuration",
    items: [
      { title: "Auth Config", href: "/docs/configuration/auth-config" },
      { title: "Cookie Config", href: "/docs/configuration/cookie-config" },
      { title: "Email Config", href: "/docs/configuration/email-config" },
    ],
  },
  {
    title: "API",
    items: [
      { title: "Signup", href: "/docs/api/signup" },
      { title: "Login", href: "/docs/api/login" },
      { title: "Logout", href: "/docs/api/logout" },
      { title: "Verify Email", href: "/docs/api/verify-email" },
      { title: "Reset Password", href: "/docs/api/reset-password" },
      { title: "Session", href: "/docs/api/session" },
      { title: "Sessions", href: "/docs/api/sessions" },
    ],
  },
  {
    title: "Axum",
    items: [
      { title: "Router", href: "/docs/axum/router" },
      { title: "Extractors", href: "/docs/axum/extractors" },
      { title: "Middleware", href: "/docs/axum/middleware" },
    ],
  },
  {
    title: "CLI",
    items: [{ title: "Commands", href: "/docs/cli/commands" }],
  },
  {
    title: "OAuth",
    items: [
      { title: "Google", href: "/docs/oauth/google" },
      { title: "GitHub", href: "/docs/oauth/github" },
      { title: "Account Linking", href: "/docs/oauth/account-linking" },
    ],
  },
];

export function DocsSidebar() {
  const pathname = usePathname();
  const [openSections, setOpenSections] = useState<Set<string>>(
    new Set(navSections.map((s) => s.title)),
  );

  const toggleSection = (title: string) => {
    const newOpenSections = new Set(openSections);
    if (newOpenSections.has(title)) {
      newOpenSections.delete(title);
    } else {
      newOpenSections.add(title);
    }
    setOpenSections(newOpenSections);
  };

  return (
    <aside className="docs-sidebar">
      {/* Search trigger */}
      <div className="border-b border-foreground/[0.06] p-4">
        <button
          type="button"
          className="flex w-full items-center gap-2 rounded-sm border border-foreground/[0.06] bg-[var(--surface)] px-3 py-2 text-left text-sm text-[var(--muted-foreground)] transition hover:border-[var(--accent)]"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <title>Search icon</title>
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.3-4.3" />
          </svg>
          <span>Search...</span>
          <kbd className="ml-auto font-mono text-xs text-[var(--muted-foreground)]">
            ⌘K
          </kbd>
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-4">
        <div className="space-y-6">
          {navSections.map((section) => {
            const isOpen = openSections.has(section.title);
            return (
              <div key={section.title}>
                <button
                  type="button"
                  onClick={() => toggleSection(section.title)}
                  className="mb-2 flex w-full items-center justify-between font-mono text-[11px] font-medium uppercase tracking-[0.14em] text-[var(--muted-foreground)] transition hover:text-[var(--foreground)]"
                >
                  <span>{section.title}</span>
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    className={`transition-transform ${isOpen ? "rotate-180" : ""}`}
                    aria-hidden="true"
                  >
                    <title>Toggle section</title>
                    <path d="m6 9 6 6 6-6" />
                  </svg>
                </button>
                {isOpen && (
                  <div className="space-y-0.5">
                    {section.items.map((item) => {
                      const isActive = pathname === item.href;
                      return (
                        <Link
                          key={item.href}
                          href={item.href}
                          className={`block rounded-sm px-3 py-1.5 text-sm transition ${
                            isActive
                              ? "bg-[var(--surface)] font-medium text-[var(--foreground)]"
                              : "text-[var(--muted-foreground)] hover:bg-[var(--surface)] hover:text-[var(--foreground)]"
                          }`}
                        >
                          {item.title}
                        </Link>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </nav>

      {/* Footer */}
      <div className="flex items-center justify-between border-t border-foreground/[0.06] p-4">
        <a
          href="https://github.com/rs-auth/rs-auth"
          target="_blank"
          rel="noopener noreferrer"
          className="flex h-8 w-8 items-center justify-center rounded-sm border border-foreground/[0.06] text-[var(--muted-foreground)] transition hover:bg-[var(--surface)] hover:text-[var(--foreground)]"
        >
          <span className="sr-only">GitHub repository</span>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
          >
            <title>GitHub icon</title>
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
          </svg>
        </a>
        <ThemeToggle />
      </div>
    </aside>
  );
}
