use axum_extra::extract::SignedCookieJar;
use axum_lib::{Json, Router, extract::State, http::StatusCode, response::Html, routing::get};
use rs_auth_axum::{
    AuthState, auth_router,
    extract::{CurrentUser, require_current_user, resolve_optional_user},
};
use rs_auth_core::config::{CookieConfig, OAuthConfig, OAuthProviderEntry, SameSite};
use rs_auth_core::{AuthConfig, AuthService, email::LogEmailSender};
use rs_auth_postgres::{AuthDb, run_migrations};
use serde_json::json;

type App = AuthState<AuthDb, AuthDb, AuthDb, AuthDb, AuthDb, LogEmailSender>;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@127.0.0.1:5432/rs_auth_example".to_string()
    });

    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let db = AuthDb::new(pool);
    let config = build_config();
    let auth_service = AuthService::new(
        config,
        db.clone(),
        db.clone(),
        db.clone(),
        db.clone(),
        db,
        LogEmailSender,
    );
    let auth_state = AuthState::new(auth_service);

    let app = Router::new()
        .route("/", get(index))
        .route("/me", get(me))
        .route("/protected", get(protected))
        .route("/dashboard", get(dashboard))
        .nest("/auth", auth_router(auth_state.clone()))
        .with_state(auth_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum_lib::serve(listener, app).await.unwrap();
}

fn build_config() -> AuthConfig {
    AuthConfig {
        secret: std::env::var("RS_AUTH_SECRET").unwrap_or_else(|_| {
            "change-me-in-production-at-least-64-bytes-long-secret".to_string()
        }),
        cookie: CookieConfig {
            secure: false,
            same_site: SameSite::Lax,
            ..CookieConfig::default()
        },
        oauth: oauth_config_from_env(),
        ..AuthConfig::default()
    }
}

fn oauth_config_from_env() -> OAuthConfig {
    let mut providers = Vec::new();

    if let Some(p) = provider_from_env("GOOGLE", "google") {
        providers.push(p);
    }
    if let Some(p) = provider_from_env("GITHUB", "github") {
        providers.push(p);
    }

    OAuthConfig {
        providers,
        allow_implicit_account_linking: true,
        success_redirect: std::env::var("RS_AUTH_OAUTH_SUCCESS_REDIRECT")
            .ok()
            .or_else(|| Some("/dashboard".to_string())),
        error_redirect: std::env::var("RS_AUTH_OAUTH_ERROR_REDIRECT")
            .ok()
            .or_else(|| Some("/?error=oauth_login_failed".to_string())),
    }
}

fn provider_from_env(prefix: &str, provider_id: &str) -> Option<OAuthProviderEntry> {
    let client_id = std::env::var(format!("RS_AUTH_{prefix}_CLIENT_ID")).ok()?;
    let client_secret = std::env::var(format!("RS_AUTH_{prefix}_CLIENT_SECRET")).ok()?;
    let redirect_url = std::env::var(format!("RS_AUTH_{prefix}_REDIRECT_URL"))
        .unwrap_or_else(|_| format!("http://localhost:3000/auth/callback/{provider_id}"));

    Some(OAuthProviderEntry {
        provider_id: provider_id.to_string(),
        client_id,
        client_secret,
        redirect_url,
        auth_url: None,
        token_url: None,
        userinfo_url: None,
    })
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn me(
    State(state): State<App>,
    jar: SignedCookieJar,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = resolve_optional_user(&state, &jar)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(Json(json!({ "user": user.user })))
}

async fn protected(
    State(state): State<App>,
    jar: SignedCookieJar,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let CurrentUser { user, session } = require_current_user(&state, &jar)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(Json(json!({
        "message": "You have access to this protected endpoint.",
        "user_id": user.id,
        "email": user.email,
        "email_verified": user.email_verified_at.is_some(),
        "session_id": session.id,
    })))
}

async fn dashboard(
    State(state): State<App>,
    jar: SignedCookieJar,
) -> Result<Html<String>, StatusCode> {
    let CurrentUser { user, session: _ } = require_current_user(&state, &jar)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>Dashboard</title>
<style>
  body {{ font-family: sans-serif; background: #0a0a0a; color: #e5e5e5; padding: 2rem; }}
  pre {{ background: #141414; border: 1px solid #262626; border-radius: 0.5rem; padding: 1rem; }}
  a {{ color: #3b82f6; }}
</style></head><body>
<h1>Dashboard</h1>
<p>Signed in as <strong>{email}</strong></p>
<pre>{user_json}</pre>
<p><a href="/">Back to home</a> &middot; <a href="/protected">Protected API</a></p>
</body></html>"#,
        email = user.email,
        user_json = serde_json::to_string_pretty(&user).unwrap_or_default(),
    );
    Ok(Html(html))
}
