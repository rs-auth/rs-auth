use async_trait::async_trait;
use sqlx::Row;

use rs_auth_core::error::AuthError;
use rs_auth_core::store::OAuthStateStore;
use rs_auth_core::types::{NewOAuthState, OAuthState};

use crate::db::AuthDb;

#[async_trait]
impl OAuthStateStore for AuthDb {
    async fn create_oauth_state(&self, state: NewOAuthState) -> Result<OAuthState, AuthError> {
        sqlx::query(
            r#"
            INSERT INTO oauth_states (provider_id, csrf_state, pkce_verifier, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, provider_id, csrf_state, pkce_verifier, expires_at, created_at
            "#,
        )
        .bind(&state.provider_id)
        .bind(&state.csrf_state)
        .bind(&state.pkce_verifier)
        .bind(state.expires_at)
        .fetch_one(&self.pool)
        .await
        .map(|row| OAuthState {
            id: row.get("id"),
            provider_id: row.get("provider_id"),
            csrf_state: row.get("csrf_state"),
            pkce_verifier: row.get("pkce_verifier"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn find_by_csrf_state(&self, csrf_state: &str) -> Result<Option<OAuthState>, AuthError> {
        sqlx::query(
            r#"
            SELECT id, provider_id, csrf_state, pkce_verifier, expires_at, created_at
            FROM oauth_states
            WHERE csrf_state = $1
            "#,
        )
        .bind(csrf_state)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| OAuthState {
                id: row.get("id"),
                provider_id: row.get("provider_id"),
                csrf_state: row.get("csrf_state"),
                pkce_verifier: row.get("pkce_verifier"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
            })
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn delete_oauth_state(&self, id: i64) -> Result<(), AuthError> {
        sqlx::query(r#"DELETE FROM oauth_states WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn delete_expired_oauth_states(&self) -> Result<u64, AuthError> {
        sqlx::query(r#"DELETE FROM oauth_states WHERE expires_at < now()"#)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|error| AuthError::Store(error.to_string()))
    }
}
