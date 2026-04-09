use axum_extra::extract::SignedCookieJar;
use axum_lib::{Json, extract::State};
use serde::Serialize;

use crate::cookie::{get_session_token, remove_session_cookie};
use crate::error::ApiError;
use crate::state::AuthState;

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub ok: bool,
}

pub async fn logout<U, S, V, A, E>(
    State(state): State<AuthState<U, S, V, A, E>>,
    jar: SignedCookieJar,
) -> Result<(SignedCookieJar, Json<LogoutResponse>), ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let token = get_session_token(&jar, &state.config.cookie)
        .ok_or(ApiError(rs_auth_core::AuthError::SessionNotFound))?;
    let current = state.service.get_session(&token).await?;
    state.service.logout(current.session.id).await?;

    Ok((
        remove_session_cookie(jar, &state.config.cookie),
        Json(LogoutResponse { ok: true }),
    ))
}
