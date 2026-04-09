use axum_extra::extract::SignedCookieJar;
use axum_lib::extract::Request;
use axum_lib::middleware::Next;
use axum_lib::response::Response;

use crate::error::ApiError;
use crate::extract::require_current_user;
use crate::state::AuthState;

/// Middleware that requires a valid session. Injects `CurrentUser` into request extensions.
pub async fn require_auth<U, S, V, A, E>(
    state: AuthState<U, S, V, A, E>,
    jar: SignedCookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let current_user = require_current_user(&state, &jar).await?;
    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}

/// Middleware that requires a valid session with a verified email. Injects `CurrentUser` into request extensions.
pub async fn require_verified<U, S, V, A, E>(
    state: AuthState<U, S, V, A, E>,
    jar: SignedCookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let current_user = require_current_user(&state, &jar).await?;
    if current_user.user.email_verified_at.is_none() {
        return Err(ApiError(rs_auth_core::AuthError::EmailNotVerified));
    }

    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}
