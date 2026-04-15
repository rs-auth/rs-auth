#[derive(Debug, Clone)]
pub enum LoginMethod {
    Password,
    OAuth { provider_id: String },
}

#[derive(Debug, Clone)]
pub enum LoginFailReason {
    InvalidCredentials,
    EmailNotVerified,
    OAuthError,
}

#[derive(Debug, Clone)]
pub enum AuthEvent {
    UserSignedUp {
        user_id: i64,
        email: String,
    },
    UserLoggedIn {
        user_id: i64,
        method: LoginMethod,
    },
    UserLoginFailed {
        email: String,
        reason: LoginFailReason,
    },
    UserLoggedOut {
        user_id: i64,
        session_id: i64,
    },
    EmailVerified {
        user_id: i64,
    },
    PasswordResetRequested {
        user_id: i64,
    },
    PasswordResetCompleted {
        user_id: i64,
    },
    OAuthAccountLinked {
        user_id: i64,
        provider_id: String,
    },
    OAuthAccountUnlinked {
        user_id: i64,
        provider_id: String,
    },
    SessionCreated {
        user_id: i64,
        session_id: i64,
        ip: Option<String>,
    },
    SessionRevoked {
        user_id: i64,
        session_id: i64,
    },
}
