use async_trait::async_trait;

use crate::error::AuthError;
use crate::types::{
    Account, NewAccount, NewOAuthState, NewSession, NewVerification, OAuthState, Session, User,
    Verification,
};

/// Storage backend for user records.
#[async_trait]
pub trait UserStore: Send + Sync {
    async fn create_user(
        &self,
        email: &str,
        name: Option<&str>,
        password_hash: Option<&str>,
    ) -> Result<User, AuthError>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthError>;

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AuthError>;

    async fn set_email_verified(&self, user_id: i64) -> Result<(), AuthError>;

    async fn update_password(&self, user_id: i64, password_hash: &str) -> Result<(), AuthError>;

    async fn delete_user(&self, user_id: i64) -> Result<(), AuthError>;
}

/// Storage backend for session records.
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create_session(&self, session: NewSession) -> Result<Session, AuthError>;

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, AuthError>;

    async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<Session>, AuthError>;

    async fn delete_session(&self, id: i64) -> Result<(), AuthError>;

    async fn delete_by_user_id(&self, user_id: i64) -> Result<(), AuthError>;

    async fn delete_expired(&self) -> Result<u64, AuthError>;
}

/// Storage backend for verification token records.
#[async_trait]
pub trait VerificationStore: Send + Sync {
    async fn create_verification(
        &self,
        verification: NewVerification,
    ) -> Result<Verification, AuthError>;

    async fn find_by_identifier(&self, identifier: &str)
    -> Result<Option<Verification>, AuthError>;

    async fn find_by_token_hash(&self, token_hash: &str)
    -> Result<Option<Verification>, AuthError>;

    async fn delete_verification(&self, id: i64) -> Result<(), AuthError>;

    async fn delete_by_identifier(&self, identifier: &str) -> Result<(), AuthError>;

    async fn delete_expired(&self) -> Result<u64, AuthError>;
}

/// Storage backend for OAuth account records.
#[async_trait]
pub trait AccountStore: Send + Sync {
    async fn create_account(&self, account: NewAccount) -> Result<Account, AuthError>;
    async fn find_by_provider(
        &self,
        provider_id: &str,
        account_id: &str,
    ) -> Result<Option<Account>, AuthError>;
    async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<Account>, AuthError>;
    async fn delete_account(&self, id: i64) -> Result<(), AuthError>;
}

/// Storage backend for transient OAuth state records.
#[async_trait]
pub trait OAuthStateStore: Send + Sync {
    async fn create_oauth_state(&self, state: NewOAuthState) -> Result<OAuthState, AuthError>;
    async fn find_by_csrf_state(&self, csrf_state: &str) -> Result<Option<OAuthState>, AuthError>;
    async fn delete_oauth_state(&self, id: i64) -> Result<(), AuthError>;
    async fn delete_expired_oauth_states(&self) -> Result<u64, AuthError>;
}
