# rs-auth

**[rs-auth.com](https://rs-auth.com)** · [crates.io](https://crates.io/crates/rs-auth) · [docs.rs](https://docs.rs/rs-auth) · [GitHub](https://github.com/rs-auth/rs-auth)

Composable authentication for Rust, inspired by Better Auth. The `rs-auth` facade crate re-exports `rs-auth-core`, `rs-auth-postgres`, and `rs-auth-axum` for convenient access to the authentication stack.

## Current Status

**Phase 1 (Email/Password Authentication)** is complete and production-ready.

**Phase 2 (OAuth)** is available for stable Google and GitHub login and callback flows.

## Features

- Email/password signup and login
- Argon2id password hashing
- Database-backed sessions with opaque tokens (SHA-256 hashed)
- Email verification
- Password reset
- Signed cookies (via axum-extra)
- Configurable session and token TTLs
- Auto sign-in after signup
- CLI for migrations and cleanup
- OAuth login and callback for Google and GitHub

## Workspace Layout

```
rs-auth/
├── auth/          -> rs-auth (facade crate)
├── core/          -> rs-auth-core (domain logic)
├── pg/            -> rs-auth-postgres (PostgreSQL store)
├── axum/          -> rs-auth-axum (Axum handlers & router)
├── cli/           -> rs-auth-cli (CLI tool)
└── examples/
    └── basic/     -> minimal example app
```

## Quick Start

Add `rs-auth` to your `Cargo.toml`:

```toml
[dependencies]
rs-auth = "0.1"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
axum = "0.8"
axum-extra = { version = "0.10", features = ["cookie-signed"] }
tokio = { version = "1", features = ["full"] }
tracing-subscriber = "0.3"
```

Create a minimal application:

```rust
use axum_extra::extract::SignedCookieJar;
use axum::{Json, Router, extract::State, routing::get};
use rs_auth_axum::{AuthState, auth_router, extract::resolve_optional_user};
use rs_auth_core::{AuthConfig, AuthService, email::LogEmailSender};
use rs_auth_postgres::{AuthDb, run_migrations};
use serde_json::json;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/postgres".to_string());

    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let db = AuthDb::new(pool);
     let auth_service = AuthService::new(
         AuthConfig::default(),
         db.clone(),
         db.clone(),
         db.clone(),
         db.clone(),
         db,
         LogEmailSender,
     );
    let auth_state = AuthState::new(auth_service);

    let app = Router::new()
        .route("/", get(|| async { "rs-auth basic example" }))
        .route("/me", get(me))
        .nest("/auth", auth_router(auth_state.clone()))
        .with_state(auth_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn me(
    State(state): State<AuthState<AuthDb, AuthDb, AuthDb, AuthDb, AuthDb, LogEmailSender>>,
    jar: SignedCookieJar,
) -> Json<serde_json::Value> {
    let user = resolve_optional_user(&state, &jar).await.unwrap();
    Json(json!({ "user": user.user }))
}
```

## Configuration

The `AuthConfig` struct controls authentication behavior:

```rust
pub struct AuthConfig {
    pub secret: String,                    // Cookie signing secret
    pub session_ttl: Duration,             // Default: 30 days
    pub verification_ttl: Duration,        // Default: 1 hour
    pub reset_ttl: Duration,               // Default: 1 hour
    pub token_length: usize,               // Default: 32 bytes
    pub email: EmailConfig,
    pub cookie: CookieConfig,
    pub oauth: OAuthConfig,
}
```

### EmailConfig

```rust
pub struct EmailConfig {
    pub send_verification_on_signup: bool,        // Default: true
    pub require_verification_to_login: bool,      // Default: false
    pub auto_sign_in_after_signup: bool,          // Default: true
    pub auto_sign_in_after_verification: bool,    // Default: false
}
```

### CookieConfig

```rust
pub struct CookieConfig {
    pub name: String,              // Default: "rs_auth_session"
    pub http_only: bool,           // Default: true
    pub secure: bool,              // Default: true
    pub same_site: SameSite,       // Default: Lax
    pub path: String,              // Default: "/"
    pub domain: Option<String>,    // Default: None
}
```

## CLI

The `rs-auth-cli` binary provides three commands:

### Run Migrations

```bash
rs-auth-cli migrate --database-url postgres://user:pass@localhost/db
```

Creates the necessary database tables for users, sessions, verification tokens, OAuth accounts, and OAuth state.

### Generate Migration

```bash
rs-auth-cli generate <name>
```

Generates a new migration file template.

### Cleanup Expired Tokens

```bash
rs-auth-cli cleanup --database-url postgres://user:pass@localhost/db
```

Removes expired sessions, verification tokens, and OAuth state from the database.

## OAuth

Google and GitHub OAuth providers are supported. Stable endpoints are:

- `GET /auth/login/{provider}`
- `GET /auth/callback/{provider}`

Stable behavior includes:

- OAuth login
- account creation
- implicit account linking
- session creation
- JSON callback responses
- redirect-mode callback responses

Configure OAuth with `OAuthConfig`:

```rust
use rs_auth_core::{AuthConfig, OAuthConfig, OAuthProviderEntry};

let mut config = AuthConfig::default();
config.oauth = OAuthConfig {
    providers: vec![
        OAuthProviderEntry {
            provider_id: "google".to_string(),
            client_id: "your-google-client-id".to_string(),
            client_secret: "your-google-client-secret".to_string(),
            redirect_url: "http://localhost:3000/auth/callback/google".to_string(),
            auth_url: None,
            token_url: None,
            userinfo_url: None,
        },
        OAuthProviderEntry {
            provider_id: "github".to_string(),
            client_id: "your-github-client-id".to_string(),
            client_secret: "your-github-client-secret".to_string(),
            redirect_url: "http://localhost:3000/auth/callback/github".to_string(),
            auth_url: None,
            token_url: None,
            userinfo_url: None,
        },
    ],
    allow_implicit_account_linking: true,
    success_redirect: Some("/dashboard".to_string()),
    error_redirect: Some("/login?error=oauth".to_string()),
};
```

OAuth transient state is stored separately from verification tokens. Each record stores:

- `provider_id`
- `csrf_state`
- `pkce_verifier`
- `expires_at`

This keeps email/reset verification tokens isolated from OAuth login state and allows operational cleanup to handle both flows independently.

Out of scope for the current OAuth milestone:

- additional providers
- token refresh workflows
- unlinking accounts
- admin tooling
- provider management UX

## API Endpoints

The `auth_router` provides the following endpoints:

| Method | Path                      | Description                          |
|--------|---------------------------|--------------------------------------|
| POST   | `/auth/signup`            | Create a new user account            |
| POST   | `/auth/login`             | Log in with email and password       |
| POST   | `/auth/logout`            | Log out and invalidate session       |
| GET    | `/auth/session`           | Get current session information      |
| GET    | `/auth/sessions`          | List all sessions for current user   |
| GET    | `/auth/verify/{token}`    | Verify email with token              |
| POST   | `/auth/forgot`            | Request password reset               |
| POST   | `/auth/reset`             | Reset password with token            |
| GET    | `/auth/login/{provider}`  | Initiate OAuth login                 |
| GET    | `/auth/callback/{provider}` | OAuth callback handler               |

## License

Licensed under either of:

- MIT License
- Apache License, Version 2.0

at your option.
