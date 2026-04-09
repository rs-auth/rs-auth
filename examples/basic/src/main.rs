use axum_extra::extract::SignedCookieJar;
use axum_lib::{Json, Router, extract::State, routing::get};
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
    axum_lib::serve(listener, app).await.unwrap();
}

async fn me(
    State(state): State<AuthState<AuthDb, AuthDb, AuthDb, AuthDb, LogEmailSender>>,
    jar: SignedCookieJar,
) -> Json<serde_json::Value> {
    let user = resolve_optional_user(&state, &jar).await.unwrap();
    Json(json!({ "user": user.user }))
}
