use rs_auth_core::{AuthConfig, AuthService, email::LogEmailSender};
use rs_auth_postgres::AuthDb;
use sqlx::PgPool;

pub async fn run(database_url: &str) -> anyhow::Result<()> {
    let pool = PgPool::connect(database_url).await?;
    let db = AuthDb::new(pool);
    let service = AuthService::new(
        AuthConfig::default(),
        db.clone(),
        db.clone(),
        db.clone(),
        db.clone(),
        db,
        LogEmailSender,
    );
    let (sessions, verifications, oauth_states) = service
        .cleanup_expired()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!(
        "Deleted {sessions} expired sessions, {verifications} expired verifications, and {oauth_states} expired OAuth states."
    );
    Ok(())
}
