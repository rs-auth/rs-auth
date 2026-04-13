use axum_lib::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AuthState;

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct OkResponse {
    pub ok: bool,
}

pub async fn forgot_password<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<OkResponse>, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    state.service.request_password_reset(&payload.email).await?;
    Ok(Json(OkResponse { ok: true }))
}

pub async fn reset_password<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<OkResponse>, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    state
        .service
        .reset_password(&payload.token, &payload.password)
        .await?;
    Ok(Json(OkResponse { ok: true }))
}
