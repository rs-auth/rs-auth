//! PostgreSQL persistence layer for rs-auth.
//!
//! Implements the store traits from [`rs_auth_core`] using [`sqlx`] with a
//! PostgreSQL backend. Also provides an embedded migration runner.

mod account;
pub mod db;
pub mod migrate;
mod session;
mod user;
mod verification;

pub use db::AuthDb;
pub use migrate::run_migrations;
