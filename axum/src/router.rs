use axum_lib::{
    Router, middleware,
    routing::{get, post},
};

use crate::handlers;
use crate::state::AuthState;

/// Build an Axum router with all rs-auth endpoints. Mount with `.nest("/auth", auth_router(state))`.
pub fn auth_router<U, S, V, A, O, E>(
    state: AuthState<U, S, V, A, O, E>,
) -> Router<AuthState<U, S, V, A, O, E>>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let protected = Router::new()
        .route(
            "/session",
            get(handlers::session::get_session::<U, S, V, A, O, E>),
        )
        .route(
            "/sessions",
            get(handlers::session::list_sessions::<U, S, V, A, O, E>),
        )
        .route(
            "/logout",
            post(handlers::logout::logout::<U, S, V, A, O, E>),
        )
        .route(
            "/accounts",
            get(handlers::account::list_accounts::<U, S, V, A, O, E>),
        )
        .route(
            "/accounts/{id}/unlink",
            post(handlers::account::unlink_account::<U, S, V, A, O, E>),
        )
        .route(
            "/link/{provider}",
            get(handlers::oauth::oauth_link::<U, S, V, A, O, E>),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::require_auth::<U, S, V, A, O, E>,
        ));

    let public = Router::new()
        .route(
            "/signup",
            post(handlers::signup::signup::<U, S, V, A, O, E>),
        )
        .route("/login", post(handlers::login::login::<U, S, V, A, O, E>))
        .route(
            "/verify/{token}",
            get(handlers::verify::verify_email::<U, S, V, A, O, E>),
        )
        .route(
            "/forgot",
            post(handlers::reset::forgot_password::<U, S, V, A, O, E>),
        )
        .route(
            "/reset",
            post(handlers::reset::reset_password::<U, S, V, A, O, E>),
        )
        .route(
            "/login/{provider}",
            get(handlers::oauth::oauth_login::<U, S, V, A, O, E>),
        )
        .route(
            "/callback/{provider}",
            get(handlers::oauth::oauth_callback::<U, S, V, A, O, E>),
        );

    Router::new().merge(public).merge(protected)
}
