# rs-auth Example App

A minimal testable Axum application demonstrating all rs-auth features.

## Running locally

### Quick start with Docker Compose (recommended)

```bash
cp .env.example .env
docker compose up -d
cargo run
```

### Manual PostgreSQL setup

1. Start PostgreSQL:

```bash
docker run --name rs-auth-pg -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 -d postgres:16-alpine
```

2. Configure environment:

```bash
cp .env.example .env
edit .env
```

Required variables:

- `DATABASE_URL` - PostgreSQL connection string
- `RS_AUTH_SECRET` - Cookie signing secret (at least 64 bytes)

Optional variables (to test OAuth):

- `RS_AUTH_GOOGLE_CLIENT_ID`
- `RS_AUTH_GOOGLE_CLIENT_SECRET`
- `RS_AUTH_GOOGLE_REDIRECT_URL`
- `RS_AUTH_GITHUB_CLIENT_ID`
- `RS_AUTH_GITHUB_CLIENT_SECRET`
- `RS_AUTH_GITHUB_REDIRECT_URL`
- `RS_AUTH_OAUTH_SUCCESS_REDIRECT`
- `RS_AUTH_OAUTH_ERROR_REDIRECT`

3. Run:

```bash
cargo run
```

The app will be available at `http://0.0.0.0:3000`.

## Features demonstrated

- **Email/password signup and login** - Form-based auth with password hashing
- **OAuth login** - Google and GitHub OAuth buttons appear when provider env vars are set
- **Session management** - Lists active sessions and allows logout
- **Protected routes** - Middleware guards `/protected` and `/dashboard`
- **Cookie security** - Uses signed cookies with HttpOnly, Secure, SameSite=Lax

## Testing the app

### Test email/password flow

1. Visit http://localhost:3000
2. Click "Sign up"
3. Fill in email, name, and password
4. After signup, you are redirected to dashboard (auto sign-in)
5. Click "Log out"
6. Log in with the same credentials

### Test OAuth flow (requires provider setup)

1. Create OAuth apps in your Google/GitHub developer consoles
2. Set redirect URIs to `http://localhost:3000/auth/callback/{provider}`
3. Add provider credentials to `.env`
4. Restart the app
5. Visit http://localhost:3000
6. Click "Google" or "GitHub" button
7. Complete OAuth flow
8. You are redirected to `/dashboard` on success

### Test protected routes

1. Log in
2. Visit `/protected` - Returns JSON with user info
3. Log out
4. Visit `/protected` - Returns 401 Unauthorized

### Test session management

1. Log in from two different browsers
2. Visit `/me` in each browser - Shows sessions list
3. Click "Log out" - Removes session cookie
4. Verify session no longer works

## Architecture

This example uses:

- `AuthService` - Core authentication logic
- `AuthDb` - PostgreSQL storage
- `LogEmailSender` - Prints verification/reset tokens to console
- `SignedCookieJar` - Cookie signing and validation
- `CurrentUser` extractor - Middleware for protected routes
- `require_auth` middleware - Enforces authentication

### Database schema

Migrations are auto-run on startup using `run_migrations`. The schema includes:

- `users` - User accounts
- `sessions` - Session tokens
- `verifications` - Email verification and password reset tokens
- `oauth_states` - OAuth CSRF/PKCE state (dedicated table)
- `accounts` - Linked OAuth accounts
