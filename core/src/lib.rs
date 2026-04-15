//! Core domain logic for rs-auth.
//!
//! This crate contains configuration types, error definitions, store traits,
//! the email sender trait, cryptographic helpers, and the [`AuthService`] that
//! drives all authentication flows.

pub mod config;
pub mod email;
pub mod error;
pub mod events;
pub mod hooks;
pub mod oauth;
pub mod rate_limit;
pub mod service;
pub mod store;
pub mod types;

pub mod crypto {
    pub mod hash;
    pub mod token;
}

pub use config::{AuthConfig, CookieConfig, EmailConfig, SameSite};
pub use error::AuthError;
pub use events::{AuthEvent, LoginFailReason, LoginMethod};
pub use hooks::{AuthHook, EventEmitter};
pub use rate_limit::{NoOpRateLimiter, RateLimitAction, RateLimiter};
pub use service::{
    AuthService, LinkAccountResult, LoginResult, RefreshTokenResult, RequestResetResult,
    ResetPasswordResult, SessionResult, SignupResult, UnlinkAccountResult, VerifyEmailResult,
};
pub use store::OAuthStateStore;
pub use types::{OAuthIntent, PublicAccount};
