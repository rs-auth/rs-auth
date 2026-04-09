use rs_auth_postgres::run_migrations;
use sqlx::PgPool;

pub async fn run(database_url: &str) -> anyhow::Result<()> {
    let pool = PgPool::connect(database_url).await?;
    run_migrations(&pool).await?;
    Ok(())
}
