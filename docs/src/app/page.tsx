import Link from "next/link";
import { ServerCodeBlock } from "fumadocs-ui/components/codeblock.rsc";
import { Nav } from "@/components/nav";
import { HeroCanvas } from "@/components/hero-canvas";
import vesperLight from "@/themes/vesper-light.json";

const codeThemes = {
  light: vesperLight as never,
  dark: "vesper" as const,
} as const;

export default function Home() {
  return (
    <>
      <Nav />
      <div className="home-page relative pt-[var(--landing-topbar-height)] lg:pt-0">
        <div className="relative text-foreground">
          <div className="home-shell flex flex-col lg:flex-row">
            {/* Left side — Hero (sticky on desktop) */}
            <div className="home-left relative w-full lg:w-[40%] lg:min-h-[calc(100dvh-var(--landing-topbar-height))] border-b lg:border-b-0 lg:border-r border-foreground/[0.06] px-5 sm:px-6 lg:px-7 lg:sticky lg:top-[var(--landing-topbar-height)] z-10 bg-background overflow-hidden">
              {/* Dithering warp canvas — fills entire left panel */}
              <HeroCanvas className="absolute inset-0 z-0 pointer-events-none" />
              <div className="relative z-10 flex flex-col justify-end lg:!justify-center h-full min-h-[60vh] lg:min-h-[calc(100dvh-var(--landing-topbar-height))] pb-10 lg:pb-0">
                {/* Hero content */}
                <div className="mt-auto lg:mt-0 space-y-6 max-w-lg lg:-mt-8">
                  <div className="inline-flex w-fit items-center gap-2 border border-foreground/[0.15] bg-foreground/[0.03] px-3 py-1.5 font-mono text-[10px] uppercase tracking-wider text-foreground/65">
                    <span className="text-foreground/35">01</span>
                    <span>Axum + Postgres + OAuth</span>
                  </div>
                  <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl lg:text-[2.75rem] lg:leading-[1.15] xl:text-5xl">
                    Composable authentication for Rust
                  </h1>
                  <p className="text-base leading-7 text-foreground/60 lg:text-[15px]">
                    Built around Axum and Postgres with custom sessions,
                    email/password flows, OAuth, and a CLI for migrations.
                  </p>

                  <div className="flex flex-wrap gap-3 pt-2">
                    <Link
                      href="/docs"
                      className="inline-flex items-center gap-1.5 bg-foreground text-background px-5 py-2.5 font-mono text-xs uppercase tracking-wider transition hover:opacity-90"
                    >
                      Get Started
                    </Link>
                    <a
                      href="https://github.com/rs-auth/rs-auth"
                      target="_blank"
                      rel="noreferrer"
                      className="inline-flex items-center gap-1.5 border border-foreground/[0.15] px-5 py-2.5 font-mono text-xs uppercase tracking-wider text-foreground/70 transition hover:text-foreground hover:border-foreground/20"
                    >
                      GitHub
                      <svg
                        className="h-2.5 w-2.5 opacity-50"
                        viewBox="0 0 10 10"
                        fill="none"
                      >
                        <title>External link</title>
                        <path
                          d="M1 9L9 1M9 1H3M9 1V7"
                          stroke="currentColor"
                          strokeWidth="1.2"
                        />
                      </svg>
                    </a>
                  </div>
                </div>
              </div>
            </div>

            {/* Right side — README content (scrolls) */}
            <div className="home-right relative z-0 w-full lg:w-[60%] overflow-x-hidden">
              <div className="px-5 sm:px-8 lg:px-10 py-10 lg:!pt-20 lg:pb-16">
                {/* README heading */}
                <div className="flex items-center gap-4 font-monospace text-sm uppercase tracking-wider text-foreground/80 mb-6">
                  <span>README</span>
                  <hr className="w-full" />
                </div>

                <p className="text-[15px] leading-7 text-foreground/70 max-w-2xl">
                  rs-auth is an authentication library for Rust. It provides
                  email/password auth, database-backed sessions, email
                  verification, password reset, and OAuth — all designed for
                  Axum and PostgreSQL.
                </p>

                {/* Install command */}
                <div className="mt-8">
                  <ServerCodeBlock
                    lang="toml"
                    code={`[dependencies]\nrs-auth = "0.1"`}
                    themes={codeThemes}
                    defaultColor={false}
                    codeblock={{ title: "Cargo.toml", className: "my-0" }}
                  />
                </div>

                {/* Features section */}
                <div className="flex items-center gap-4 font-mono text-muted-foreground text-xs uppercase tracking-wider text- mt-14 mb-6">
                  <span>Features</span>
                  <hr className="w-full" />
                </div>

                <div className="grid gap-px sm:grid-cols-2 lg:grid-cols-2 border border-foreground/[0.08]">
                  <FeatureCard
                    num="01"
                    title="Email & Password"
                    headline="Built-in credential auth."
                    description="Signup, login, logout with secure password hashing, email verification, and password reset."
                  />
                  <FeatureCard
                    num="02"
                    title="Sessions"
                    headline="Database-backed sessions."
                    description="Opaque tokens stored in Postgres with signed cookies and configurable expiry."
                  />
                  <FeatureCard
                    num="03"
                    title="Axum Integration"
                    headline="Framework-native."
                    description="Router, extractors, middleware, and a prebuilt auth handler you nest into your app."
                  />
                  <FeatureCard
                    num="04"
                    title="OAuth"
                    headline="Social sign-on."
                    description="Google and GitHub flows with automatic account linking and state verification."
                  />
                  <FeatureCard
                    num="05"
                    title="PostgreSQL"
                    headline="SQLx-backed storage."
                    description="Migrations, session tracking, user management, and OAuth account persistence."
                  />
                  <FeatureCard
                    num="06"
                    title="CLI"
                    headline="Developer tooling."
                    description="Run migrations, manage configuration, and scaffold auth setup from the command line."
                  />
                </div>

                {/* Code example */}
                <div className="flex items-center gap-4 font-mono text-muted-foreground text-xs uppercase tracking-wider text- mt-14 mb-6">
                  <span className="min-w-full">Quick Start</span>
                  <hr className="w-full" />
                </div>

                <div className="mt-2">
                  <ServerCodeBlock
                    lang="rust"
                    code={`use axum::Router;
use rs_auth::axum::{auth_router, AuthState};
use rs_auth::config::AuthConfig;

let config = AuthConfig::builder()
    .database_url(env::var("DATABASE_URL")?)
    .cookie_secret(env::var("COOKIE_SECRET")?)
    .build();

let state = AuthState::new(config).await?;
let auth = auth_router(state.clone());

let app = Router::new()
    .nest("/auth", auth)
    .with_state(state);

let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
    .await?;
axum::serve(listener, app).await?;`}
                    themes={codeThemes}
                    defaultColor={false}
                    codeblock={{ title: "src/main.rs", className: "my-0" }}
                  />
                </div>

                {/* Architecture */}
                <div className="flex items-center gap-4 font-mono text-muted-foreground text-xs uppercase tracking-wider text- mt-14 mb-6">
                  <span>Architecture</span>
                  <hr className="w-full" />
                </div>

                <p className="text-[15px] leading-7 text-foreground/70 max-w-2xl mb-6">
                  rs-auth is a Cargo workspace split into focused crates. Use
                  only what you need, or pull in the facade crate for
                  everything.
                </p>

                <div className="grid gap-px sm:grid-cols-2 border border-foreground/[0.08]">
                  <CrateCard
                    name="rs-auth"
                    description="Facade crate that re-exports everything."
                  />
                  <CrateCard
                    name="rs-auth-core"
                    description="Auth logic, password hashing, token generation."
                  />
                  <CrateCard
                    name="rs-auth-postgres"
                    description="SQLx-backed user, session, and OAuth storage."
                  />
                  <CrateCard
                    name="rs-auth-axum"
                    description="Axum router, extractors, and middleware."
                  />
                </div>

                {/* Bottom CTA */}
                <div className="mt-14 border border-foreground/[0.08] p-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                  <div>
                    <p className="font-mono text-xs uppercase tracking-wider text- mb-1">
                      Get started
                    </p>
                    <p className="text-[15px] text-foreground/70">
                      Read the docs, install the crate, and ship auth in
                      minutes.
                    </p>
                  </div>
                  <Link
                    href="/docs"
                    className="inline-flex items-center gap-1.5 shrink-0 bg-foreground text-background px-5 py-2.5 font-mono text-xs uppercase tracking-wider transition hover:opacity-90"
                  >
                    Open Docs
                  </Link>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

function FeatureCard({
  num,
  title,
  headline,
  description,
}: {
  num: string;
  title: string;
  headline: string;
  description: string;
}) {
  return (
    <div className="p-5 bg-background border-b border-r border-foreground/[0.08] last:border-b-0 sm:[&:nth-last-child(-n+2)]:border-b-0">
      <div className="flex items-baseline gap-2 mb-1">
        <span className="font-mono text-[10px] uppercase tracking-wider text-foreground/35">
          {num}
        </span>
        <span className="font-mono text-xs uppercase tracking-wider text-foreground/70">
          {title}
        </span>
      </div>
      <p className="text-[15px] font-medium text-foreground mb-1">{headline}</p>
      <p className="text-sm leading-6 text-">{description}</p>
    </div>
  );
}

function CrateCard({
  name,
  description,
}: {
  name: string;
  description: string;
}) {
  return (
    <div className="p-4 bg-background border-b border-r border-foreground/[0.08] last:border-b-0 sm:[&:nth-last-child(-n+2)]:border-b-0">
      <code className="font-mono text-sm text-foreground">{name}</code>
      <p className="text-sm leading-6 text- mt-1">{description}</p>
    </div>
  );
}
