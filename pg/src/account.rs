use async_trait::async_trait;
use rs_auth_core::error::AuthError;
use rs_auth_core::store::AccountStore;
use rs_auth_core::types::{Account, NewAccount};
use sqlx::Row;

use crate::db::AuthDb;

#[async_trait]
impl AccountStore for AuthDb {
    async fn create_account(&self, account: NewAccount) -> Result<Account, AuthError> {
        let row = sqlx::query(
            r#"
            INSERT INTO accounts (user_id, provider_id, account_id, access_token, refresh_token, access_token_expires_at, scope)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, provider_id, account_id, access_token, refresh_token, access_token_expires_at, scope, created_at, updated_at
            "#,
        )
        .bind(account.user_id)
        .bind(&account.provider_id)
        .bind(&account.account_id)
        .bind(&account.access_token)
        .bind(&account.refresh_token)
        .bind(account.access_token_expires_at)
        .bind(&account.scope)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::Store(e.to_string()))?;

        Ok(Account {
            id: row.get("id"),
            user_id: row.get("user_id"),
            provider_id: row.get("provider_id"),
            account_id: row.get("account_id"),
            access_token: row.get("access_token"),
            refresh_token: row.get("refresh_token"),
            access_token_expires_at: row.get("access_token_expires_at"),
            scope: row.get("scope"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn find_by_provider(
        &self,
        provider_id: &str,
        account_id: &str,
    ) -> Result<Option<Account>, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, provider_id, account_id, access_token, refresh_token, access_token_expires_at, scope, created_at, updated_at
            FROM accounts
            WHERE provider_id = $1 AND account_id = $2
            "#,
        )
        .bind(provider_id)
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Store(e.to_string()))?;

        Ok(row.map(|row| Account {
            id: row.get("id"),
            user_id: row.get("user_id"),
            provider_id: row.get("provider_id"),
            account_id: row.get("account_id"),
            access_token: row.get("access_token"),
            refresh_token: row.get("refresh_token"),
            access_token_expires_at: row.get("access_token_expires_at"),
            scope: row.get("scope"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<Account>, AuthError> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, provider_id, account_id, access_token, refresh_token, access_token_expires_at, scope, created_at, updated_at
            FROM accounts
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::Store(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| Account {
                id: row.get("id"),
                user_id: row.get("user_id"),
                provider_id: row.get("provider_id"),
                account_id: row.get("account_id"),
                access_token: row.get("access_token"),
                refresh_token: row.get("refresh_token"),
                access_token_expires_at: row.get("access_token_expires_at"),
                scope: row.get("scope"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn delete_account(&self, id: i64) -> Result<(), AuthError> {
        sqlx::query(r#"DELETE FROM accounts WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::Store(e.to_string()))?;
        Ok(())
    }
}
