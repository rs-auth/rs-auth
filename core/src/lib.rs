//! Core domain logic for rs-auth.
//!
//! This crate contains configuration types, error definitions, store traits,
//! the email sender trait, cryptographic helpers, and the [`AuthService`] that
//! drives all authentication flows.

pub mod config;
pub mod email;
pub mod error;
pub mod oauth;
pub mod service;
pub mod store;
pub mod types;

pub mod crypto {
    pub mod hash;
    pub mod token;
}

pub use config::{AuthConfig, CookieConfig, EmailConfig, SameSite};
pub use error::AuthError;
pub use service::{
    AuthService, LoginResult, RequestResetResult, ResetPasswordResult, SessionResult, SignupResult,
    VerifyEmailResult,
};
pub use store::OAuthStateStore;
