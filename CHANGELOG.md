# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-04-09

### Added

#### rs-auth-core
- `AuthConfig` with configurable session, cookie, email, and OAuth settings
- `AuthService` with full email/password auth flows
- `UserStore`, `SessionStore`, `VerificationStore`, `AccountStore` traits
- Argon2id password hashing
- SHA-256 session token hashing
- Email verification and password reset token generation
- `EmailSender` trait with `LogEmailSender` for development
- OAuth primitives: PKCE, CSRF state, token exchange, Google and GitHub user info
- `OAuthConfig` with `success_redirect` and `error_redirect` support

#### rs-auth-postgres
- SQLx-backed implementations of all store traits
- PostgreSQL schema migrations for users, sessions, verifications, and OAuth accounts
- `AuthDb` struct implementing all four store traits over a shared `PgPool`
- `run_migrations` helper

#### rs-auth-axum
- `auth_router` mounting all auth endpoints
- `AuthState` wrapping `AuthService` for Axum state injection
- Cookie-based session management with signed cookies via `axum-extra`
- `CurrentUser` and `OptionalUser` extractors
- `RequireVerified` middleware
- Handlers: signup, login, logout, session, sessions, verify email, forgot password, reset password
- OAuth handlers: login redirect and callback for Google and GitHub
- `error_redirect` honored on all OAuth failure paths

#### rs-auth (facade)
- Re-exports `rs-auth-core`, `rs-auth-postgres`, and `rs-auth-axum`
- Feature flags: `postgres` (default), `axum` (default)

#### rs-auth-cli
- `migrate` — runs database migrations
- `generate <name>` — scaffolds a new migration file
- `cleanup` — removes expired sessions and verification tokens

[0.1.0]: https://github.com/rs-auth/rs-auth/releases/tag/v0.1.0
