use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct AuthDb {
    pub pool: PgPool,
}

impl AuthDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
