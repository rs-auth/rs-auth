use axum_extra::extract::SignedCookieJar;
use axum_lib::{Json, extract::State};
use rs_auth_core::types::{PublicUser, Session};
use serde::Serialize;

use crate::cookie::get_session_token;
use crate::error::ApiError;
use crate::state::AuthState;

#[derive(Debug, Serialize)]
pub struct CurrentSessionResponse {
    pub user: PublicUser,
    pub session: Session,
}

#[derive(Debug, Serialize)]
pub struct SessionsResponse {
    pub sessions: Vec<Session>,
}

pub async fn get_session<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    jar: SignedCookieJar,
) -> Result<Json<CurrentSessionResponse>, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let token = get_session_token(&jar, &state.config.cookie)
        .ok_or(ApiError(rs_auth_core::AuthError::SessionNotFound))?;
    let current = state.service.get_session(&token).await?;

    Ok(Json(CurrentSessionResponse {
        user: PublicUser::from(&current.user),
        session: current.session,
    }))
}

pub async fn list_sessions<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    jar: SignedCookieJar,
) -> Result<Json<SessionsResponse>, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let token = get_session_token(&jar, &state.config.cookie)
        .ok_or(ApiError(rs_auth_core::AuthError::SessionNotFound))?;
    let current = state.service.get_session(&token).await?;
    let sessions = state.service.list_sessions(current.user.id).await?;

    Ok(Json(SessionsResponse { sessions }))
}
