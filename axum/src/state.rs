use std::sync::Arc;

use rs_auth_core::email::EmailSender;
use rs_auth_core::store::{AccountStore, SessionStore, UserStore, VerificationStore};
use rs_auth_core::{AuthConfig, AuthService};

use crate::cookie::make_cookie_key;

/// Shared application state for Axum handlers. Wraps an [`AuthService`] in an `Arc` and carries the config and cookie signing key.
pub struct AuthState<U, S, V, A, E>
where
    U: UserStore,
    S: SessionStore,
    V: VerificationStore,
    A: AccountStore,
    E: EmailSender,
{
    pub service: Arc<AuthService<U, S, V, A, E>>,
    pub config: AuthConfig,
    pub key: axum_extra::extract::cookie::Key,
}

impl<U, S, V, A, E> Clone for AuthState<U, S, V, A, E>
where
    U: UserStore,
    S: SessionStore,
    V: VerificationStore,
    A: AccountStore,
    E: EmailSender,
{
    fn clone(&self) -> Self {
        Self {
            service: Arc::clone(&self.service),
            config: self.config.clone(),
            key: self.key.clone(),
        }
    }
}

impl<U, S, V, A, E> AuthState<U, S, V, A, E>
where
    U: UserStore,
    S: SessionStore,
    V: VerificationStore,
    A: AccountStore,
    E: EmailSender,
{
    /// Create a new `AuthState` from an `AuthService`.
    pub fn new(service: AuthService<U, S, V, A, E>) -> Self {
        let config = service.config.clone();
        let key = make_cookie_key(&service.config.secret);
        Self {
            service: Arc::new(service),
            config,
            key,
        }
    }
}

impl<U, S, V, A, E> axum_lib::extract::FromRef<AuthState<U, S, V, A, E>>
    for axum_extra::extract::cookie::Key
where
    U: UserStore,
    S: SessionStore,
    V: VerificationStore,
    A: AccountStore,
    E: EmailSender,
{
    fn from_ref(state: &AuthState<U, S, V, A, E>) -> Self {
        state.key.clone()
    }
}
