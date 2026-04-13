use axum_extra::extract::SignedCookieJar;
use axum_lib::extract::FromRequestParts;
use axum_lib::http::request::Parts;
use rs_auth_core::types::{PublicUser, Session};

use crate::cookie::get_session_token;
use crate::error::ApiError;
use crate::state::AuthState;

/// Extractor for the currently authenticated user. Fails if no valid session exists.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    /// The authenticated user's public information.
    pub user: PublicUser,
    /// The current session.
    pub session: Session,
}

/// Extractor for an optionally authenticated user. Always succeeds, with `None` if no session.
#[derive(Debug, Clone, Default)]
pub struct OptionalUser {
    /// The authenticated user's public information, if any.
    pub user: Option<PublicUser>,
    /// The current session, if any.
    pub session: Option<Session>,
}

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or(ApiError(rs_auth_core::AuthError::SessionNotFound))
    }
}

impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .map(|current_user| OptionalUser {
                user: Some(current_user.user),
                session: Some(current_user.session),
            })
            .unwrap_or_default())
    }
}

impl<S> FromRequestParts<S> for ClientInfo
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to get IP from X-Forwarded-For or X-Real-IP headers
        let ip = parts
            .headers
            .get("x-forwarded-for")
            .or_else(|| parts.headers.get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

        let user_agent = parts
            .headers
            .get(axum_lib::http::header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Ok(ClientInfo { ip, user_agent })
    }
}

/// Resolve the current user from the session cookie. Returns an error if no valid session exists.
pub async fn require_current_user<U, S, V, A, O, E>(
    state: &AuthState<U, S, V, A, O, E>,
    jar: &SignedCookieJar,
) -> Result<CurrentUser, ApiError>
where
    U: rs_auth_core::store::UserStore,
    S: rs_auth_core::store::SessionStore,
    V: rs_auth_core::store::VerificationStore,
    A: rs_auth_core::store::AccountStore,
    O: rs_auth_core::store::OAuthStateStore,
    E: rs_auth_core::email::EmailSender,
{
    let token = get_session_token(jar, &state.config.cookie)
        .ok_or(ApiError(rs_auth_core::AuthError::SessionNotFound))?;
    let current = state.service.get_session(&token).await?;

    Ok(CurrentUser {
        user: PublicUser::from(&current.user),
        session: current.session,
    })
}

/// Resolve the current user from the session cookie. Returns `None` if no valid session exists.
pub async fn resolve_optional_user<U, S, V, A, O, E>(
    state: &AuthState<U, S, V, A, O, E>,
    jar: &SignedCookieJar,
) -> Result<OptionalUser, ApiError>
where
    U: rs_auth_core::store::UserStore,
    S: rs_auth_core::store::SessionStore,
    V: rs_auth_core::store::VerificationStore,
    A: rs_auth_core::store::AccountStore,
    O: rs_auth_core::store::OAuthStateStore,
    E: rs_auth_core::email::EmailSender,
{
    let Some(token) = get_session_token(jar, &state.config.cookie) else {
        return Ok(OptionalUser::default());
    };

    match state.service.get_session(&token).await {
        Ok(current) => Ok(OptionalUser {
            user: Some(PublicUser::from(&current.user)),
            session: Some(current.session),
        }),
        Err(rs_auth_core::AuthError::SessionNotFound) => Ok(OptionalUser::default()),
        Err(error) => Err(ApiError(error)),
    }
}
