use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// User record with all fields including password hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user ID.
    pub id: i64,
    /// User's email address.
    pub email: String,
    /// User's display name.
    pub name: Option<String>,
    /// Hashed password. None for OAuth-only users.
    pub password_hash: Option<String>,
    /// Timestamp when email was verified.
    pub email_verified_at: Option<OffsetDateTime>,
    /// User's profile image URL.
    pub image: Option<String>,
    /// Timestamp when user was created.
    pub created_at: OffsetDateTime,
    /// Timestamp when user was last updated.
    pub updated_at: OffsetDateTime,
}

impl User {
    pub fn is_verified(&self) -> bool {
        self.email_verified_at.is_some()
    }
}

/// Public user record without sensitive fields like password hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    /// Unique user ID.
    pub id: i64,
    /// User's email address.
    pub email: String,
    /// User's display name.
    pub name: Option<String>,
    /// Timestamp when email was verified.
    pub email_verified_at: Option<OffsetDateTime>,
    /// User's profile image URL.
    pub image: Option<String>,
    /// Timestamp when user was created.
    pub created_at: OffsetDateTime,
}

impl From<User> for PublicUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            email_verified_at: user.email_verified_at,
            image: user.image,
            created_at: user.created_at,
        }
    }
}

impl From<&User> for PublicUser {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            email_verified_at: user.email_verified_at,
            image: user.image.clone(),
            created_at: user.created_at,
        }
    }
}

/// Input for creating a new user.
#[derive(Debug, Clone, Deserialize)]
pub struct NewUser {
    /// User's email address.
    pub email: String,
    /// User's display name.
    pub name: Option<String>,
    /// User's plaintext password.
    pub password: String,
}

/// Session record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID.
    pub id: i64,
    /// Hashed session token.
    pub token_hash: String,
    /// User ID this session belongs to.
    pub user_id: i64,
    /// Timestamp when session expires.
    pub expires_at: OffsetDateTime,
    /// IP address where session was created.
    pub ip_address: Option<String>,
    /// User agent where session was created.
    pub user_agent: Option<String>,
    /// Timestamp when session was created.
    pub created_at: OffsetDateTime,
    /// Timestamp when session was last updated.
    pub updated_at: OffsetDateTime,
}

/// Input for creating a new session.
#[derive(Debug, Clone)]
pub struct NewSession {
    /// Hashed session token.
    pub token_hash: String,
    /// User ID this session belongs to.
    pub user_id: i64,
    /// Timestamp when session expires.
    pub expires_at: OffsetDateTime,
    /// IP address where session was created.
    pub ip_address: Option<String>,
    /// User agent where session was created.
    pub user_agent: Option<String>,
}

/// Verification token record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    /// Unique verification ID.
    pub id: i64,
    /// Identifier for the verification (e.g., "email-verify:user@example.com").
    pub identifier: String,
    /// Hashed verification token.
    pub token_hash: String,
    /// Timestamp when verification expires.
    pub expires_at: OffsetDateTime,
    /// Timestamp when verification was created.
    pub created_at: OffsetDateTime,
    /// Timestamp when verification was last updated.
    pub updated_at: OffsetDateTime,
}

/// Input for creating a new verification token.
#[derive(Debug, Clone)]
pub struct NewVerification {
    /// Identifier for the verification (e.g., "email-verify:user@example.com").
    pub identifier: String,
    /// Hashed verification token.
    pub token_hash: String,
    /// Timestamp when verification expires.
    pub expires_at: OffsetDateTime,
}

// ---------- Account ----------

/// OAuth account record linking a user to an external provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique account ID.
    pub id: i64,
    /// User ID this account belongs to.
    pub user_id: i64,
    /// OAuth provider identifier (e.g., "google", "github").
    pub provider_id: String,
    /// Account ID from the OAuth provider.
    pub account_id: String,
    /// OAuth access token.
    pub access_token: Option<String>,
    /// OAuth refresh token.
    pub refresh_token: Option<String>,
    /// Timestamp when access token expires.
    pub access_token_expires_at: Option<OffsetDateTime>,
    /// OAuth scope granted.
    pub scope: Option<String>,
    /// Timestamp when account was created.
    pub created_at: OffsetDateTime,
    /// Timestamp when account was last updated.
    pub updated_at: OffsetDateTime,
}

/// Input for creating a new OAuth account.
#[derive(Debug, Clone)]
pub struct NewAccount {
    /// User ID this account belongs to.
    pub user_id: i64,
    /// OAuth provider identifier (e.g., "google", "github").
    pub provider_id: String,
    /// Account ID from the OAuth provider.
    pub account_id: String,
    /// OAuth access token.
    pub access_token: Option<String>,
    /// OAuth refresh token.
    pub refresh_token: Option<String>,
    /// Timestamp when access token expires.
    pub access_token_expires_at: Option<OffsetDateTime>,
    /// OAuth scope granted.
    pub scope: Option<String>,
}
