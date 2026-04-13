use axum_extra::extract::SignedCookieJar;
use axum_lib::{Json, extract::State, http::HeaderMap};
use rs_auth_core::types::PublicUser;
use serde::{Deserialize, Serialize};

use crate::cookie::set_session_cookie;
use crate::error::ApiError;
use crate::state::AuthState;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: PublicUser,
}

pub async fn login<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    jar: SignedCookieJar,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<(SignedCookieJar, Json<LoginResponse>), ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let result = state
        .service
        .login(
            &payload.email,
            &payload.password,
            super::signup::forwarded_ip(&headers),
            super::signup::user_agent(&headers),
        )
        .await?;

    Ok((
        set_session_cookie(
            jar,
            &result.session_token,
            &state.config.cookie,
            state.config.session_ttl,
        ),
        Json(LoginResponse {
            user: PublicUser::from(result.user),
        }),
    ))
}
