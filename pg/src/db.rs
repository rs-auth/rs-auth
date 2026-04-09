use sqlx::PgPool;

/// PostgreSQL database wrapper for rs-auth storage backends.
#[derive(Clone, Debug)]
pub struct AuthDb {
    /// SQLx PostgreSQL connection pool.
    pub pool: PgPool,
}

impl AuthDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
