use axum_extra::extract::SignedCookieJar;
use axum_lib::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use rs_auth_core::types::PublicUser;
use serde::Serialize;

use crate::cookie::set_session_cookie;
use crate::error::ApiError;
use crate::state::AuthState;

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub user: PublicUser,
}

pub async fn verify_email<U, S, V, A, E>(
    State(state): State<AuthState<U, S, V, A, E>>,
    jar: SignedCookieJar,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Result<(SignedCookieJar, Json<VerifyResponse>), ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let result = state
        .service
        .verify_email(
            &token,
            super::signup::forwarded_ip(&headers),
            super::signup::user_agent(&headers),
        )
        .await?;

    let jar = match result.session_token.as_deref() {
        Some(token) => {
            set_session_cookie(jar, token, &state.config.cookie, state.config.session_ttl)
        }
        None => jar,
    };

    Ok((
        jar,
        Json(VerifyResponse {
            user: PublicUser::from(result.user),
        }),
    ))
}
