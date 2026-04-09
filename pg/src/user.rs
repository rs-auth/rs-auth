use async_trait::async_trait;
use sqlx::Row;

use rs_auth_core::error::AuthError;
use rs_auth_core::store::UserStore;
use rs_auth_core::types::User;

use crate::db::AuthDb;

#[async_trait]
impl UserStore for AuthDb {
    async fn create_user(
        &self,
        email: &str,
        name: Option<&str>,
        password_hash: Option<&str>,
    ) -> Result<User, AuthError> {
        sqlx::query(
            r#"
            INSERT INTO users (email, name, password_hash)
            VALUES (LOWER($1), $2, $3)
            RETURNING id, email, name, password_hash, email_verified_at, image, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map(|row| User {
            id: row.get("id"),
            email: row.get("email"),
            name: row.get("name"),
            password_hash: row.get("password_hash"),
            email_verified_at: row.get("email_verified_at"),
            image: row.get("image"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .map_err(|error| match error {
            sqlx::Error::Database(database_error) if database_error.is_unique_violation() => {
                AuthError::EmailTaken
            }
            other => AuthError::Store(other.to_string()),
        })
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthError> {
        sqlx::query(
            r#"
            SELECT id, email, name, password_hash, email_verified_at, image, created_at, updated_at
            FROM users
            WHERE LOWER(email) = LOWER($1)
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| User {
                id: row.get("id"),
                email: row.get("email"),
                name: row.get("name"),
                password_hash: row.get("password_hash"),
                email_verified_at: row.get("email_verified_at"),
                image: row.get("image"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AuthError> {
        sqlx::query(
            r#"
            SELECT id, email, name, password_hash, email_verified_at, image, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| User {
                id: row.get("id"),
                email: row.get("email"),
                name: row.get("name"),
                password_hash: row.get("password_hash"),
                email_verified_at: row.get("email_verified_at"),
                image: row.get("image"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn set_email_verified(&self, user_id: i64) -> Result<(), AuthError> {
        sqlx::query(
            r#"UPDATE users SET email_verified_at = now(), updated_at = now() WHERE id = $1"#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn update_password(&self, user_id: i64, password_hash: &str) -> Result<(), AuthError> {
        sqlx::query(r#"UPDATE users SET password_hash = $1, updated_at = now() WHERE id = $2"#)
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| AuthError::Store(error.to_string()))
    }

    async fn delete_user(&self, user_id: i64) -> Result<(), AuthError> {
        sqlx::query(r#"DELETE FROM users WHERE id = $1"#)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| AuthError::Store(error.to_string()))
    }
}
