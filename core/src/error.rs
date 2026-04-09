use thiserror::Error;

/// Authentication error types.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("email already in use")]
    EmailTaken,

    #[error("user not found")]
    UserNotFound,

    #[error("session not found or expired")]
    SessionNotFound,

    #[error("token invalid or expired")]
    InvalidToken,

    #[error("email not verified")]
    EmailNotVerified,

    #[error("password must be at least {0} characters")]
    WeakPassword(usize),

    #[error("hash error: {0}")]
    Hash(String),

    #[error("store error: {0}")]
    Store(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("oauth error: {0}")]
    OAuth(String),
}
