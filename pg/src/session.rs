use async_trait::async_trait;
use sqlx::Row;

use rs_auth_core::error::AuthError;
use rs_auth_core::store::SessionStore;
use rs_auth_core::types::{NewSession, Session};

use crate::db::AuthDb;

#[async_trait]
impl SessionStore for AuthDb {
    async fn create_session(&self, session: NewSession) -> Result<Session, AuthError> {
        sqlx::query(
            r#"
            INSERT INTO sessions (token_hash, user_id, expires_at, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, token_hash, user_id, expires_at, ip_address, user_agent, created_at, updated_at
            "#,
        )
        .bind(session.token_hash)
        .bind(session.user_id)
        .bind(session.expires_at)
        .bind(session.ip_address)
        .bind(session.user_agent)
        .fetch_one(&self.pool)
        .await
        .map(|row| Session {
            id: row.get("id"),
            token_hash: row.get("token_hash"),
            user_id: row.get("user_id"),
            expires_at: row.get("expires_at"),
            ip_address: row.get("ip_address"),
            user_agent: row.get("user_agent"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, AuthError> {
        sqlx::query(
            r#"
            SELECT id, token_hash, user_id, expires_at, ip_address, user_agent, created_at, updated_at
            FROM sessions
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| Session {
                id: row.get("id"),
                token_hash: row.get("token_hash"),
                user_id: row.get("user_id"),
                expires_at: row.get("expires_at"),
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<Session>, AuthError> {
        sqlx::query(
            r#"
            SELECT id, token_hash, user_id, expires_at, ip_address, user_agent, created_at, updated_at
            FROM sessions
            WHERE user_id = $1 AND expires_at > now()
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| Session {
                    id: row.get("id"),
                    token_hash: row.get("token_hash"),
                    user_id: row.get("user_id"),
                    expires_at: row.get("expires_at"),
                    ip_address: row.get("ip_address"),
                    user_agent: row.get("user_agent"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                })
                .collect()
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn delete_session(&self, id: i64) -> Result<(), AuthError> {
        sqlx::query(r#"DELETE FROM sessions WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn delete_by_user_id(&self, user_id: i64) -> Result<(), AuthError> {
        sqlx::query(r#"DELETE FROM sessions WHERE user_id = $1"#)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn delete_expired(&self) -> Result<u64, AuthError> {
        sqlx::query(r#"DELETE FROM sessions WHERE expires_at < now()"#)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|error| AuthError::Store(error.to_string()))
    }
}
