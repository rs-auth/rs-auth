use async_trait::async_trait;
use sqlx::Row;

use rs_auth_core::error::AuthError;
use rs_auth_core::store::VerificationStore;
use rs_auth_core::types::{NewVerification, Verification};

use crate::db::AuthDb;

#[async_trait]
impl VerificationStore for AuthDb {
    async fn create_verification(
        &self,
        verification: NewVerification,
    ) -> Result<Verification, AuthError> {
        sqlx::query(
            r#"
            INSERT INTO verifications (identifier, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, identifier, token_hash, expires_at, created_at, updated_at
            "#,
        )
        .bind(verification.identifier)
        .bind(verification.token_hash)
        .bind(verification.expires_at)
        .fetch_one(&self.pool)
        .await
        .map(|row| Verification {
            id: row.get("id"),
            identifier: row.get("identifier"),
            token_hash: row.get("token_hash"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn find_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<Verification>, AuthError> {
        sqlx::query(
            r#"
            SELECT id, identifier, token_hash, expires_at, created_at, updated_at
            FROM verifications
            WHERE identifier = $1
            "#,
        )
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| Verification {
                id: row.get("id"),
                identifier: row.get("identifier"),
                token_hash: row.get("token_hash"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Verification>, AuthError> {
        sqlx::query(
            r#"
            SELECT id, identifier, token_hash, expires_at, created_at, updated_at
            FROM verifications
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| Verification {
                id: row.get("id"),
                identifier: row.get("identifier"),
                token_hash: row.get("token_hash"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn delete_verification(&self, id: i64) -> Result<(), AuthError> {
        sqlx::query(r#"DELETE FROM verifications WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn delete_by_identifier(&self, identifier: &str) -> Result<(), AuthError> {
        sqlx::query(r#"DELETE FROM verifications WHERE identifier = $1"#)
            .bind(identifier)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn delete_expired(&self) -> Result<u64, AuthError> {
        sqlx::query(r#"DELETE FROM verifications WHERE expires_at < now()"#)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|error| AuthError::Store(error.to_string()))
    }
}
