use async_trait::async_trait;

use crate::error::AuthError;
use crate::types::User;

/// Trait for sending authentication-related emails.
#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_verification_email(&self, user: &User, token: &str) -> Result<(), AuthError>;

    async fn send_password_reset_email(&self, user: &User, token: &str) -> Result<(), AuthError>;
}

/// Development email sender that logs emails instead of sending them.
#[derive(Debug, Default, Clone, Copy)]
pub struct LogEmailSender;

#[async_trait]
impl EmailSender for LogEmailSender {
    async fn send_verification_email(&self, user: &User, token: &str) -> Result<(), AuthError> {
        tracing::info!(email = %user.email, token = %token, "verification email (dev mode)");
        Ok(())
    }

    async fn send_password_reset_email(&self, user: &User, token: &str) -> Result<(), AuthError> {
        tracing::info!(email = %user.email, token = %token, "password reset email (dev mode)");
        Ok(())
    }
}
