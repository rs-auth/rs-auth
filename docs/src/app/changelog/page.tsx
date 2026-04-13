import { History } from "lucide-react";
import Link from "next/link";
import { HeroCanvas } from "@/components/hero-canvas";
import { HomeShell } from "@/components/home-shell";
import { Nav } from "@/components/nav";
import { ChangelogContent } from "./changelog-content";

export const dynamic = "force-static";

interface GitHubRelease {
  id: number;
  tag_name: string;
  name: string;
  body: string;
  html_url: string;
  prerelease: boolean;
  published_at: string;
}

export default async function ChangelogPage() {
  let releases: GitHubRelease[] = [];
  try {
    const res = await fetch(
        "https://api.github.com/repos/rs-auth/rs-auth/releases",
        {
          next: { revalidate: 300 },
          headers: {
            Accept: "application/vnd.github.v3+json",
          },
      },
    );
    if (res.ok) {
      releases = await res.json();
    } else {
      console.error(`Changelog fetch failed: ${res.status}`);
    }
  } catch (e) {
    console.error("Changelog fetch failed:", e);
  }

  const EXPANDABLE_LINE_THRESHOLD = 15;

  const messages = releases
    ?.filter((release) => !release.prerelease)
    .map((release) => {
      const content = release.body ?? "";
      const lineCount = content
        .split("\n")
        .filter((l) => l.trim().length > 0).length;
      return {
        tag: release.tag_name,
        title: release.name,
        content,
        date: new Date(release.published_at).toLocaleDateString("en-US", {
          year: "numeric",
          month: "short",
          day: "numeric",
        }),
        url: release.html_url,
        expandable: lineCount > EXPANDABLE_LINE_THRESHOLD,
      };
    });

  return (
    <>
      <Nav />
      <HomeShell
        leftClassName="relative w-full lg:w-[40%] lg:min-h-[calc(100dvh-var(--landing-topbar-height))] border-b lg:border-b-0 lg:border-r border-foreground/[0.06] px-5 sm:px-6 lg:px-7 lg:sticky lg:top-[var(--landing-topbar-height)] z-10 bg-background overflow-hidden"
        rightClassName="relative z-0 w-full lg:w-[60%] overflow-x-hidden"
        left={
          <>
            <HeroCanvas className="absolute inset-0 z-0 pointer-events-none opacity-60" />
            <div className="relative z-10 flex flex-col justify-end lg:!justify-center h-full min-h-[60vh] lg:min-h-[calc(100dvh-var(--landing-topbar-height))] pb-10 lg:pb-0">
              <div className="mt-auto lg:mt-0 space-y-6 max-w-lg lg:-mt-8">
                <div className="inline-flex w-fit items-center gap-2 border border-foreground/[0.15] bg-foreground/[0.03] px-3 py-1.5 font-mono text-[10px] uppercase tracking-wider text-foreground/65">
                  <span className="text-foreground/35">02</span>
                  <span>GitHub Releases</span>
                </div>

                <div className="space-y-3">
                  <div className="flex items-center gap-1.5 text-foreground/60">
                    <History
                      className="h-[0.9em] w-[0.9em]"
                      aria-hidden="true"
                    />
                    <span className="text-sm">Changelog</span>
                  </div>
                  <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl lg:text-[2.75rem] lg:leading-[1.15] xl:text-5xl">
                    All changes, fixes, and updates
                  </h1>
                  <p className="text-base leading-7 text-foreground/60 lg:text-[15px] max-w-md">
                    Every release shipped to rs-auth, pulled directly from
                    GitHub and rendered here.
                  </p>
                </div>

                <div className="border-t border-foreground/10 pt-4 space-y-0 max-w-md">
                  <div className="flex items-baseline justify-between py-1.5 border-b border-dashed border-foreground/[0.06]">
                    <span className="text-xs text-foreground/70 uppercase tracking-wider">
                      Latest
                    </span>
                    <span className="text-xs text-foreground/85 font-mono">
                      {messages?.[0]?.tag ?? "\u2014"}
                    </span>
                  </div>
                </div>

                <div className="flex flex-wrap gap-3 pt-2">
                  <Link
                    href="https://github.com/rs-auth/rs-auth/releases"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1.5 bg-foreground text-background px-5 py-2.5 font-mono text-xs uppercase tracking-wider transition hover:opacity-90"
                  >
                    GitHub Releases
                  </Link>
                  <Link
                    href="/docs"
                    className="inline-flex items-center gap-1.5 border border-foreground/[0.15] px-5 py-2.5 font-mono text-xs uppercase tracking-wider text-foreground/70 transition hover:text-foreground hover:border-foreground/20"
                  >
                    Open Docs
                  </Link>
                </div>
              </div>
            </div>
          </>
        }
        right={
          <div className="px-5 sm:px-8 lg:px-10 py-10 lg:!pt-20 lg:pb-16">
            <div className="flex items-center gap-4 font-mono text-sm uppercase tracking-wider text-foreground/80 mb-6">
              <span>Changelog</span>
              <hr className="w-full" />
            </div>

            <ChangelogContent messages={messages ?? []} />
          </div>
        }
      />
    </>
  );
}

export const metadata = {
  title: "Changelog",
  description: "Latest changes, fixes, and updates to rs-auth",
};
