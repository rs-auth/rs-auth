//! Axum HTTP integration for rs-auth.
//!
//! Provides handlers, extractors, middleware, cookie helpers, and a
//! pre-built [`auth_router`] for mounting authentication endpoints.

pub mod cookie;
pub mod error;
pub mod extract;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod state;

pub use router::auth_router;
pub use state::AuthState;
