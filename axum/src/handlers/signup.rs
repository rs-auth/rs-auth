use axum_extra::extract::SignedCookieJar;
use axum_lib::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use rs_auth_core::types::{NewUser, PublicUser};
use serde::{Deserialize, Serialize};

use crate::cookie::set_session_cookie;
use crate::error::ApiError;
use crate::state::AuthState;

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignupResponse {
    pub user: PublicUser,
}

pub async fn signup<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    jar: SignedCookieJar,
    headers: HeaderMap,
    Json(payload): Json<SignupRequest>,
) -> Result<(SignedCookieJar, (StatusCode, Json<SignupResponse>)), ApiError>
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
        .signup(
            NewUser {
                email: payload.email,
                name: payload.name,
                password: payload.password,
            },
            forwarded_ip(&headers),
            user_agent(&headers),
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
        (
            StatusCode::CREATED,
            Json(SignupResponse {
                user: PublicUser::from(result.user),
            }),
        ),
    ))
}

pub(crate) fn forwarded_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(',').next().unwrap_or(value).trim().to_string())
}

pub(crate) fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum_lib::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}
