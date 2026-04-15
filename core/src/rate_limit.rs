use async_trait::async_trait;

use crate::error::AuthError;

#[derive(Debug, Clone)]
pub enum RateLimitAction {
    Signup,
    Login,
    PasswordReset,
    OAuthCallback,
}

#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check(&self, action: RateLimitAction, key: &str) -> Result<(), AuthError> {
        let _ = (action, key);
        Ok(())
    }
}

pub struct NoOpRateLimiter;

#[async_trait]
impl RateLimiter for NoOpRateLimiter {}
