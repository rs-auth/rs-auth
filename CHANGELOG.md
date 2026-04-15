# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-04-15

### Added

- Email/password signup with configurable auto-sign-in and email verification.
- Login with credential validation and optional email-verification requirement.
- Session management: create, list, delete, delete-all, delete-expired.
- Email verification flow with configurable auto-sign-in on verification.
- Password reset flow (request + reset) with token expiry.
- OAuth 2.0 login via Google and GitHub with PKCE and CSRF protection.
- Implicit account linking when OAuth email matches an existing user.
- `AuthService` generic over storage backends (`UserStore`, `SessionStore`, `VerificationStore`, `AccountStore`, `OAuthStateStore`) and `EmailSender`.
- `rs-auth-postgres` crate with SQLx-based PostgreSQL persistence and embedded migrations.
- `rs-auth-axum` crate with Axum handlers, router, middleware (`require_auth`, `require_verified`), and cookie-based session management.
- `rs-auth-cli` crate for administrative tasks (migrate, cleanup).
- `rs-auth` facade crate that re-exports `rs-auth-core`, `rs-auth-postgres`, and `rs-auth-axum` behind feature flags.
- CI workflow (check, test with PostgreSQL 16, clippy, rustfmt).
