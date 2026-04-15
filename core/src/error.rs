use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OAuthError {
    #[error("unknown provider: {provider}")]
    ProviderNotFound { provider: String },

    #[error("unsupported provider: {provider}")]
    UnsupportedProvider { provider: String },

    #[error("oauth provider misconfigured: {message}")]
    Misconfigured { message: String },

    #[error("oauth state invalid or expired")]
    InvalidState,

    #[error("oauth token exchange failed")]
    ExchangeFailed,

    #[error("oauth userinfo request failed")]
    UserInfoFailed,

    #[error("oauth userinfo payload invalid")]
    UserInfoMalformed,

    #[error("oauth provider did not return an access token")]
    MissingAccessToken,

    #[error("oauth provider did not provide a usable email")]
    MissingEmail,

    #[error("account linking by email is disabled")]
    LinkingDisabled,

    #[error("account not found")]
    AccountNotFound,

    #[error("cannot unlink last authentication method")]
    LastAuthMethod,

    #[error("account already linked to a different user")]
    AccountAlreadyLinked,

    #[error("token refresh failed")]
    RefreshFailed,

    #[error("no refresh token available")]
    NoRefreshToken,
}

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
    OAuth(#[from] OAuthError),
}
