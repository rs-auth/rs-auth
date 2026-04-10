//! Composable authentication for Rust.
//!
//! `rs-auth` is a facade crate that re-exports [`rs_auth_core`] (always),
//! [`rs_auth_postgres`] (behind the `postgres` feature), and [`rs_auth_axum`]
//! (behind the `axum` feature).
//!
//! # Quick start
//!
//! ```toml
//! [dependencies]
//! rs-auth = { version = "0.1", features = ["postgres", "axum"] }
//! ```

pub use rs_auth_core as core;
pub use rs_auth_core::*;

#[cfg(feature = "postgres")]
pub use rs_auth_postgres as postgres;

#[cfg(feature = "axum")]
pub use rs_auth_axum as axum;
