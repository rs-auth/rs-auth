use axum_extra::extract::SignedCookieJar;
use axum_lib::extract::FromRequestParts;
use axum_lib::extract::{Request, State};
use axum_lib::middleware::Next;
use axum_lib::response::{IntoResponse, Response};

use crate::error::ApiError;
use crate::extract::require_current_user;
use crate::state::AuthState;

/// Middleware that requires a valid session. Injects `CurrentUser` into request extensions.
pub async fn require_auth<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    request: Request,
    next: Next,
) -> Response
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let (mut parts, body) = request.into_parts();
    let jar = match SignedCookieJar::from_request_parts(&mut parts, &state).await {
        Ok(jar) => jar,
        Err(_) => return ApiError(rs_auth_core::AuthError::SessionNotFound).into_response(),
    };

    let current_user = match require_current_user(&state, &jar).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };

    let mut request = Request::from_parts(parts, body);
    request.extensions_mut().insert(current_user);
    next.run(request).await
}

/// Middleware that requires a valid session with a verified email. Injects `CurrentUser` into request extensions.
pub async fn require_verified<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    request: Request,
    next: Next,
) -> Response
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let (mut parts, body) = request.into_parts();
    let jar = match SignedCookieJar::from_request_parts(&mut parts, &state).await {
        Ok(jar) => jar,
        Err(_) => return ApiError(rs_auth_core::AuthError::SessionNotFound).into_response(),
    };

    let current_user = match require_current_user(&state, &jar).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };

    if current_user.user.email_verified_at.is_none() {
        return ApiError(rs_auth_core::AuthError::EmailNotVerified).into_response();
    }

    let mut request = Request::from_parts(parts, body);
    request.extensions_mut().insert(current_user);
    next.run(request).await
}
