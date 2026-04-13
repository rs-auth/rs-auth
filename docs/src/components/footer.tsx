import Link from "next/link";
import { ThemeToggle } from "@/components/theme-toggle";

const footerLinks = [
  { label: "Readme", href: "/" },
  { label: "Docs", href: "/docs" },
  { label: "Changelog", href: "/changelog" },
  {
    label: "Crates.io",
    href: "https://crates.io/crates/rs-auth",
    external: true,
  },
];

export function Footer() {
  return (
    <footer className="relative px-5 sm:px-6 lg:px-8 py-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-x-1 gap-y-1.5">
          {footerLinks.map((link, index) => (
            <span key={link.label} className="flex items-center">
              <Link
                href={link.href}
                target={link.external ? "_blank" : undefined}
                rel={link.external ? "noopener noreferrer" : undefined}
                className="inline-flex items-center gap-1 text-[11px] font-mono text-foreground/50 transition-colors hover:text-foreground/80"
              >
                {link.label}
              </Link>
              {index < footerLinks.length - 1 && (
                <span className="mx-1 select-none text-[10px] text-foreground/10">
                  /
                </span>
              )}
            </span>
          ))}
        </div>

        <div className="flex w-full shrink-0 items-center justify-between sm:w-auto sm:gap-4">
          <span className="font-mono text-[10px] text-foreground/50">
            © {new Date().getFullYear()} rs-auth
          </span>
          <div className="flex items-center gap-3 sm:gap-4">
            <span className="hidden select-none text-foreground/10 sm:inline">
              ·
            </span>
            <Link
              href="https://twitter.com/rs_auth"
              aria-label="Twitter"
              target="_blank"
              rel="noopener noreferrer"
              className="text-foreground/50 transition-colors hover:text-foreground/80"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                className="h-3.5 w-3.5"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d="M18.901 1.153h3.68l-8.04 9.19L24 22.847h-7.406l-5.8-7.584l-6.638 7.584H.474l8.6-9.83L0 1.154h7.594l5.243 6.932zM17.61 20.645h2.039L6.486 3.24H4.298z" />
              </svg>
            </Link>
            <Link
              href="https://github.com/rs-auth/rs-auth"
              aria-label="GitHub"
              target="_blank"
              rel="noopener noreferrer"
              className="text-foreground/50 transition-colors hover:text-foreground/80"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                className="h-3.5 w-3.5"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d="M12 0C5.373 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387c.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416c-.546-1.387-1.333-1.756-1.333-1.756c-1.089-.745.083-.729.083-.729c1.205.084 1.839 1.237 1.839 1.237c1.07 1.834 2.807 1.304 3.492.997c.107-.775.418-1.305.762-1.604c-2.665-.305-5.467-1.334-5.467-5.931c0-1.311.469-2.381 1.236-3.221c-.124-.303-.535-1.524.117-3.176c0 0 1.008-.322 3.301 1.23A11.52 11.52 0 0 1 12 6.844c1.02.005 2.047.138 3.006.404c2.291-1.552 3.297-1.23 3.297-1.23c.653 1.653.242 2.874.118 3.176c.77.84 1.235 1.911 1.235 3.221c0 4.609-2.807 5.624-5.479 5.921c.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.565 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12" />
              </svg>
            </Link>
            <div className="h-4 w-4 select-none text-foreground/15">|</div>
            <div className="-ml-4 sm:-ml-5">
              <ThemeToggle />
            </div>
          </div>
        </div>
      </div>
    </footer>
  );
}
