use std::sync::Arc;

use time::OffsetDateTime;

use crate::config::AuthConfig;
use crate::crypto::{hash, token};
use crate::email::EmailSender;
use crate::error::AuthError;
use crate::oauth::{OAuthTokens, OAuthUserInfo};
use crate::store::{AccountStore, SessionStore, UserStore, VerificationStore};
use crate::types::{NewAccount, NewSession, NewUser, NewVerification, Session, User, Verification};

/// Result returned by [`AuthService::signup`].
#[derive(Debug)]
pub struct SignupResult {
    /// The newly created user.
    pub user: User,
    /// Session created if auto-sign-in is enabled.
    pub session: Option<Session>,
    /// Raw session token if auto-sign-in is enabled.
    pub session_token: Option<String>,
    /// Raw verification token if email verification is enabled.
    pub verification_token: Option<String>,
}

/// Result returned by [`AuthService::login`].
#[derive(Debug)]
pub struct LoginResult {
    /// The authenticated user.
    pub user: User,
    /// The newly created session.
    pub session: Session,
    /// Raw session token to be stored in a cookie.
    pub session_token: String,
}

/// Result returned by [`AuthService::verify_email`].
#[derive(Debug)]
pub struct VerifyEmailResult {
    /// The user whose email was verified.
    pub user: User,
    /// Session created if auto-sign-in after verification is enabled.
    pub session: Option<Session>,
    /// Raw session token if auto-sign-in after verification is enabled.
    pub session_token: Option<String>,
}

/// Result returned by [`AuthService::request_password_reset`].
#[derive(Debug, Default)]
pub struct RequestResetResult {
    /// Private field to prevent construction outside this crate.
    pub _private: (),
}

/// Result returned by [`AuthService::reset_password`].
#[derive(Debug)]
pub struct ResetPasswordResult {
    /// The user whose password was reset.
    pub user: User,
}

/// Result returned by [`AuthService::get_session`].
#[derive(Debug)]
pub struct SessionResult {
    /// The user associated with the session.
    pub user: User,
    /// The session details.
    pub session: Session,
}

/// Core authentication service. Generic over storage backends and email sender.
pub struct AuthService<U, S, V, A, E>
where
    U: UserStore,
    S: SessionStore,
    V: VerificationStore,
    A: AccountStore,
    E: EmailSender,
{
    /// Authentication configuration.
    pub config: AuthConfig,
    /// User storage backend.
    pub users: Arc<U>,
    /// Session storage backend.
    pub sessions: Arc<S>,
    /// Verification token storage backend.
    pub verifications: Arc<V>,
    /// OAuth account storage backend.
    pub accounts: Arc<A>,
    /// Email sender implementation.
    pub email: Arc<E>,
}

impl<U, S, V, A, E> AuthService<U, S, V, A, E>
where
    U: UserStore,
    S: SessionStore,
    V: VerificationStore,
    A: AccountStore,
    E: EmailSender,
{
    /// Create a new authentication service with the given configuration and backends.
    pub fn new(
        config: AuthConfig,
        users: U,
        sessions: S,
        verifications: V,
        accounts: A,
        email: E,
    ) -> Self {
        Self {
            config,
            users: Arc::new(users),
            sessions: Arc::new(sessions),
            verifications: Arc::new(verifications),
            accounts: Arc::new(accounts),
            email: Arc::new(email),
        }
    }

    /// Register a new user with email and password.
    pub async fn signup(
        &self,
        input: NewUser,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<SignupResult, AuthError> {
        if input.password.len() < 8 {
            return Err(AuthError::WeakPassword(8));
        }

        let email = input.email.trim().to_lowercase();
        if self.users.find_by_email(&email).await?.is_some() {
            return Err(AuthError::EmailTaken);
        }

        let password_hash = hash::hash_password(&input.password)?;
        let user = self
            .users
            .create_user(&email, input.name.as_deref(), Some(&password_hash))
            .await?;

        let verification_token = if self.config.email.send_verification_on_signup {
            let identifier = format!("email-verify:{}", user.email.to_lowercase());
            let raw_token = token::generate_token(self.config.token_length);

            let _ = self.verifications.delete_by_identifier(&identifier).await;
            self.verifications
                .create_verification(NewVerification {
                    identifier,
                    token_hash: token::hash_token(&raw_token),
                    expires_at: OffsetDateTime::now_utc() + self.config.verification_ttl,
                })
                .await?;
            self.email
                .send_verification_email(&user, &raw_token)
                .await?;
            Some(raw_token)
        } else {
            None
        };

        let (session, session_token) = if self.config.email.auto_sign_in_after_signup {
            let (session, raw_token) = self
                .create_session_internal(user.id, ip, user_agent)
                .await?;
            (Some(session), Some(raw_token))
        } else {
            (None, None)
        };

        Ok(SignupResult {
            user,
            session,
            session_token,
            verification_token,
        })
    }

    /// Authenticate a user with email and password.
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<LoginResult, AuthError> {
        let user = self
            .users
            .find_by_email(&email.trim().to_lowercase())
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let password_hash = user
            .password_hash
            .as_deref()
            .ok_or(AuthError::InvalidCredentials)?;
        if !hash::verify_password(password, password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        if self.config.email.require_verification_to_login && !user.is_verified() {
            return Err(AuthError::EmailNotVerified);
        }

        let (session, session_token) = self
            .create_session_internal(user.id, ip, user_agent)
            .await?;

        Ok(LoginResult {
            user,
            session,
            session_token,
        })
    }

    /// Delete a single session by ID.
    pub async fn logout(&self, session_id: i64) -> Result<(), AuthError> {
        self.sessions.delete_session(session_id).await
    }

    /// Delete all sessions for a user.
    pub async fn logout_all(&self, user_id: i64) -> Result<(), AuthError> {
        self.sessions.delete_by_user_id(user_id).await
    }

    /// Retrieve a session and its associated user by raw token.
    pub async fn get_session(&self, raw_token: &str) -> Result<SessionResult, AuthError> {
        let session = self
            .sessions
            .find_by_token_hash(&token::hash_token(raw_token))
            .await?
            .ok_or(AuthError::SessionNotFound)?;

        if session.expires_at < OffsetDateTime::now_utc() {
            self.sessions.delete_session(session.id).await?;
            return Err(AuthError::SessionNotFound);
        }

        let user = self
            .users
            .find_by_id(session.user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(SessionResult { user, session })
    }

    /// List all active sessions for a user.
    pub async fn list_sessions(&self, user_id: i64) -> Result<Vec<Session>, AuthError> {
        self.sessions.find_by_user_id(user_id).await
    }

    /// Verify a user's email address using a verification token.
    pub async fn verify_email(
        &self,
        raw_token: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<VerifyEmailResult, AuthError> {
        let verification = self.lookup_verification(raw_token, "email-verify:").await?;
        let email = verification
            .identifier
            .strip_prefix("email-verify:")
            .ok_or(AuthError::InvalidToken)?;

        let user = self
            .users
            .find_by_email(email)
            .await?
            .ok_or(AuthError::UserNotFound)?;
        self.users.set_email_verified(user.id).await?;
        self.verifications
            .delete_verification(verification.id)
            .await?;

        let user = self
            .users
            .find_by_id(user.id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        let (session, session_token) = if self.config.email.auto_sign_in_after_verification {
            let (session, raw_token) = self
                .create_session_internal(user.id, ip, user_agent)
                .await?;
            (Some(session), Some(raw_token))
        } else {
            (None, None)
        };

        Ok(VerifyEmailResult {
            user,
            session,
            session_token,
        })
    }

    /// Request a password reset token for a user by email.
    pub async fn request_password_reset(
        &self,
        email: &str,
    ) -> Result<RequestResetResult, AuthError> {
        let email = email.trim().to_lowercase();
        if let Some(user) = self.users.find_by_email(&email).await? {
            let identifier = format!("password-reset:{}", user.email.to_lowercase());
            let _ = self.verifications.delete_by_identifier(&identifier).await;

            let raw_token = token::generate_token(self.config.token_length);
            self.verifications
                .create_verification(NewVerification {
                    identifier,
                    token_hash: token::hash_token(&raw_token),
                    expires_at: OffsetDateTime::now_utc() + self.config.reset_ttl,
                })
                .await?;
            self.email
                .send_password_reset_email(&user, &raw_token)
                .await?;
        }

        Ok(RequestResetResult::default())
    }

    /// Reset a user's password using a reset token.
    pub async fn reset_password(
        &self,
        raw_token: &str,
        new_password: &str,
    ) -> Result<ResetPasswordResult, AuthError> {
        if new_password.len() < 8 {
            return Err(AuthError::WeakPassword(8));
        }

        let verification = self
            .lookup_verification(raw_token, "password-reset:")
            .await?;
        let email = verification
            .identifier
            .strip_prefix("password-reset:")
            .ok_or(AuthError::InvalidToken)?;
        let user = self
            .users
            .find_by_email(email)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        self.users
            .update_password(user.id, &hash::hash_password(new_password)?)
            .await?;
        self.sessions.delete_by_user_id(user.id).await?;
        self.verifications
            .delete_verification(verification.id)
            .await?;

        let user = self
            .users
            .find_by_id(user.id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(ResetPasswordResult { user })
    }

    /// Delete expired sessions and verification tokens. Returns (sessions_deleted, verifications_deleted).
    pub async fn cleanup_expired(&self) -> Result<(u64, u64), AuthError> {
        let sessions_deleted = self.sessions.delete_expired().await?;
        let verifications_deleted = self.verifications.delete_expired().await?;
        Ok((sessions_deleted, verifications_deleted))
    }

    /// Handle OAuth callback - find or create user from OAuth info.
    ///
    /// NOTE: OAuth state verification happens in the handler layer before calling this method.
    /// The CSRF state and PKCE verifier are stored in the `verifications` table with the
    /// identifier format `oauth-state:{csrf_token}`. This reuses existing infrastructure
    /// rather than requiring a dedicated OAuth state table.
    pub async fn oauth_callback(
        &self,
        info: OAuthUserInfo,
        tokens: OAuthTokens,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<LoginResult, AuthError> {
        // 1. Check if account already exists for this provider
        if let Some(account) = self
            .accounts
            .find_by_provider(&info.provider_id, &info.account_id)
            .await?
        {
            // Existing account - just create session
            let user = self
                .users
                .find_by_id(account.user_id)
                .await?
                .ok_or(AuthError::UserNotFound)?;
            let (session, session_token) = self
                .create_session_internal(user.id, ip, user_agent)
                .await?;
            return Ok(LoginResult {
                user,
                session,
                session_token,
            });
        }

        // 2. Check if user with this email already exists
        let user = if let Some(existing_user) = self.users.find_by_email(&info.email).await? {
            // Check if implicit account linking is allowed
            if !self.config.oauth.allow_implicit_account_linking {
                return Err(AuthError::OAuth(
                    "Account linking by email is disabled. Please sign in with your password first.".to_string()
                ));
            }
            existing_user
        } else {
            // 3. Create new user (no password for OAuth-only users)
            self.users
                .create_user(&info.email, info.name.as_deref(), None)
                .await?
        };

        // 4. Link account with tokens
        let access_token_expires_at = tokens.expires_in.map(|d| OffsetDateTime::now_utc() + d);
        self.accounts
            .create_account(NewAccount {
                user_id: user.id,
                provider_id: info.provider_id,
                account_id: info.account_id,
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
                access_token_expires_at,
                scope: tokens.scope,
            })
            .await?;

        // 5. Mark email as verified (OAuth providers verify emails)
        if !user.is_verified() {
            self.users.set_email_verified(user.id).await?;
        }

        // 6. Reload user and create session
        let user = self
            .users
            .find_by_id(user.id)
            .await?
            .ok_or(AuthError::UserNotFound)?;
        let (session, session_token) = self
            .create_session_internal(user.id, ip, user_agent)
            .await?;
        Ok(LoginResult {
            user,
            session,
            session_token,
        })
    }

    async fn create_session_internal(
        &self,
        user_id: i64,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(Session, String), AuthError> {
        let raw_token = token::generate_token(self.config.token_length);
        let session = self
            .sessions
            .create_session(NewSession {
                token_hash: token::hash_token(&raw_token),
                user_id,
                expires_at: OffsetDateTime::now_utc() + self.config.session_ttl,
                ip_address: ip,
                user_agent,
            })
            .await?;

        Ok((session, raw_token))
    }

    async fn lookup_verification(
        &self,
        raw_token: &str,
        prefix: &str,
    ) -> Result<Verification, AuthError> {
        let verification = self
            .verifications
            .find_by_token_hash(&token::hash_token(raw_token))
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if !verification.identifier.starts_with(prefix) {
            return Err(AuthError::InvalidToken);
        }

        if verification.expires_at < OffsetDateTime::now_utc() {
            self.verifications
                .delete_verification(verification.id)
                .await?;
            return Err(AuthError::InvalidToken);
        }

        Ok(verification)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use time::OffsetDateTime;

    use super::AuthService;
    use crate::config::AuthConfig;
    use crate::email::EmailSender;
    use crate::error::AuthError;
    use crate::oauth::OAuthTokens;
    use crate::store::{AccountStore, SessionStore, UserStore, VerificationStore};
    use crate::types::{
        Account, NewAccount, NewSession, NewUser, NewVerification, Session, User, Verification,
    };

    #[derive(Default)]
    struct MemoryState {
        next_user_id: i64,
        next_session_id: i64,
        next_verification_id: i64,
        next_account_id: i64,
        users: HashMap<i64, User>,
        sessions: HashMap<i64, Session>,
        verifications: HashMap<i64, Verification>,
        accounts: HashMap<i64, Account>,
    }

    #[derive(Clone, Default)]
    struct MemoryStore {
        inner: Arc<Mutex<MemoryState>>,
    }

    #[async_trait]
    impl UserStore for MemoryStore {
        async fn create_user(
            &self,
            email: &str,
            name: Option<&str>,
            password_hash: Option<&str>,
        ) -> Result<User, AuthError> {
            let mut state = self.inner.lock().unwrap();
            if state.users.values().any(|user| user.email == email) {
                return Err(AuthError::EmailTaken);
            }

            state.next_user_id += 1;
            let now = OffsetDateTime::now_utc();
            let user = User {
                id: state.next_user_id,
                email: email.to_string(),
                name: name.map(str::to_owned),
                password_hash: password_hash.map(str::to_owned),
                email_verified_at: None,
                image: None,
                created_at: now,
                updated_at: now,
            };
            state.users.insert(user.id, user.clone());
            Ok(user)
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthError> {
            let state = self.inner.lock().unwrap();
            Ok(state
                .users
                .values()
                .find(|user| user.email == email)
                .cloned())
        }

        async fn find_by_id(&self, id: i64) -> Result<Option<User>, AuthError> {
            Ok(self.inner.lock().unwrap().users.get(&id).cloned())
        }

        async fn set_email_verified(&self, user_id: i64) -> Result<(), AuthError> {
            let mut state = self.inner.lock().unwrap();
            let user = state
                .users
                .get_mut(&user_id)
                .ok_or(AuthError::UserNotFound)?;
            user.email_verified_at = Some(OffsetDateTime::now_utc());
            user.updated_at = OffsetDateTime::now_utc();
            Ok(())
        }

        async fn update_password(
            &self,
            user_id: i64,
            password_hash: &str,
        ) -> Result<(), AuthError> {
            let mut state = self.inner.lock().unwrap();
            let user = state
                .users
                .get_mut(&user_id)
                .ok_or(AuthError::UserNotFound)?;
            user.password_hash = Some(password_hash.to_string());
            user.updated_at = OffsetDateTime::now_utc();
            Ok(())
        }

        async fn delete_user(&self, user_id: i64) -> Result<(), AuthError> {
            self.inner.lock().unwrap().users.remove(&user_id);
            Ok(())
        }
    }

    #[async_trait]
    impl SessionStore for MemoryStore {
        async fn create_session(&self, session: NewSession) -> Result<Session, AuthError> {
            let mut state = self.inner.lock().unwrap();
            state.next_session_id += 1;
            let now = OffsetDateTime::now_utc();
            let session = Session {
                id: state.next_session_id,
                token_hash: session.token_hash,
                user_id: session.user_id,
                expires_at: session.expires_at,
                ip_address: session.ip_address,
                user_agent: session.user_agent,
                created_at: now,
                updated_at: now,
            };
            state.sessions.insert(session.id, session.clone());
            Ok(session)
        }

        async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, AuthError> {
            let state = self.inner.lock().unwrap();
            Ok(state
                .sessions
                .values()
                .find(|session| session.token_hash == token_hash)
                .cloned())
        }

        async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<Session>, AuthError> {
            let state = self.inner.lock().unwrap();
            Ok(state
                .sessions
                .values()
                .filter(|session| session.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn delete_session(&self, id: i64) -> Result<(), AuthError> {
            self.inner.lock().unwrap().sessions.remove(&id);
            Ok(())
        }

        async fn delete_by_user_id(&self, user_id: i64) -> Result<(), AuthError> {
            self.inner
                .lock()
                .unwrap()
                .sessions
                .retain(|_, session| session.user_id != user_id);
            Ok(())
        }

        async fn delete_expired(&self) -> Result<u64, AuthError> {
            let now = OffsetDateTime::now_utc();
            let mut state = self.inner.lock().unwrap();
            let before = state.sessions.len();
            state
                .sessions
                .retain(|_, session| session.expires_at >= now);
            Ok((before - state.sessions.len()) as u64)
        }
    }

    #[async_trait]
    impl VerificationStore for MemoryStore {
        async fn create_verification(
            &self,
            verification: NewVerification,
        ) -> Result<Verification, AuthError> {
            let mut state = self.inner.lock().unwrap();
            state.next_verification_id += 1;
            let now = OffsetDateTime::now_utc();
            let verification = Verification {
                id: state.next_verification_id,
                identifier: verification.identifier,
                token_hash: verification.token_hash,
                expires_at: verification.expires_at,
                created_at: now,
                updated_at: now,
            };
            state
                .verifications
                .insert(verification.id, verification.clone());
            Ok(verification)
        }

        async fn find_by_identifier(
            &self,
            identifier: &str,
        ) -> Result<Option<Verification>, AuthError> {
            let state = self.inner.lock().unwrap();
            Ok(state
                .verifications
                .values()
                .find(|verification| verification.identifier == identifier)
                .cloned())
        }

        async fn find_by_token_hash(
            &self,
            token_hash: &str,
        ) -> Result<Option<Verification>, AuthError> {
            let state = self.inner.lock().unwrap();
            Ok(state
                .verifications
                .values()
                .find(|verification| verification.token_hash == token_hash)
                .cloned())
        }

        async fn delete_verification(&self, id: i64) -> Result<(), AuthError> {
            self.inner.lock().unwrap().verifications.remove(&id);
            Ok(())
        }

        async fn delete_by_identifier(&self, identifier: &str) -> Result<(), AuthError> {
            self.inner
                .lock()
                .unwrap()
                .verifications
                .retain(|_, verification| verification.identifier != identifier);
            Ok(())
        }

        async fn delete_expired(&self) -> Result<u64, AuthError> {
            let now = OffsetDateTime::now_utc();
            let mut state = self.inner.lock().unwrap();
            let before = state.verifications.len();
            state
                .verifications
                .retain(|_, verification| verification.expires_at >= now);
            Ok((before - state.verifications.len()) as u64)
        }
    }

    #[async_trait]
    impl AccountStore for MemoryStore {
        async fn create_account(&self, account: NewAccount) -> Result<Account, AuthError> {
            let mut state = self.inner.lock().unwrap();
            state.next_account_id += 1;
            let now = OffsetDateTime::now_utc();
            let account = Account {
                id: state.next_account_id,
                user_id: account.user_id,
                provider_id: account.provider_id,
                account_id: account.account_id,
                access_token: account.access_token,
                refresh_token: account.refresh_token,
                access_token_expires_at: account.access_token_expires_at,
                scope: account.scope,
                created_at: now,
                updated_at: now,
            };
            state.accounts.insert(account.id, account.clone());
            Ok(account)
        }

        async fn find_by_provider(
            &self,
            provider_id: &str,
            account_id: &str,
        ) -> Result<Option<Account>, AuthError> {
            let state = self.inner.lock().unwrap();
            Ok(state
                .accounts
                .values()
                .find(|account| {
                    account.provider_id == provider_id && account.account_id == account_id
                })
                .cloned())
        }

        async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<Account>, AuthError> {
            let state = self.inner.lock().unwrap();
            Ok(state
                .accounts
                .values()
                .filter(|account| account.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn delete_account(&self, id: i64) -> Result<(), AuthError> {
            self.inner.lock().unwrap().accounts.remove(&id);
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct TestEmailSender {
        verification_tokens: Arc<Mutex<Vec<String>>>,
        reset_tokens: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl EmailSender for TestEmailSender {
        async fn send_verification_email(
            &self,
            _user: &User,
            token: &str,
        ) -> Result<(), AuthError> {
            self.verification_tokens
                .lock()
                .unwrap()
                .push(token.to_string());
            Ok(())
        }

        async fn send_password_reset_email(
            &self,
            _user: &User,
            token: &str,
        ) -> Result<(), AuthError> {
            self.reset_tokens.lock().unwrap().push(token.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn signup_verify_login_and_reset_flow_works() {
        let store = MemoryStore::default();
        let email = TestEmailSender::default();
        let service = AuthService::new(
            AuthConfig::default(),
            store.clone(),
            store.clone(),
            store.clone(),
            store.clone(),
            email.clone(),
        );

        let signup = service
            .signup(
                NewUser {
                    email: "test@example.com".to_string(),
                    name: Some("Test".to_string()),
                    password: "supersecret".to_string(),
                },
                Some("127.0.0.1".to_string()),
                Some("test-agent".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(signup.user.email, "test@example.com");
        assert!(signup.session.is_some());
        assert_eq!(email.verification_tokens.lock().unwrap().len(), 1);

        let verification_token = email.verification_tokens.lock().unwrap()[0].clone();
        let verify = service
            .verify_email(&verification_token, None, None)
            .await
            .unwrap();
        assert!(verify.user.is_verified());

        let login = service
            .login("test@example.com", "supersecret", None, None)
            .await
            .unwrap();
        assert_eq!(login.user.email, "test@example.com");

        service
            .request_password_reset("test@example.com")
            .await
            .unwrap();
        let reset_token = email.reset_tokens.lock().unwrap()[0].clone();
        service
            .reset_password(&reset_token, "newpassword")
            .await
            .unwrap();

        let login = service
            .login("test@example.com", "newpassword", None, None)
            .await
            .unwrap();
        assert_eq!(login.user.email, "test@example.com");
    }

    #[tokio::test]
    async fn oauth_callback_creates_new_user_and_account() {
        let store = MemoryStore::default();
        let email = TestEmailSender::default();
        let service = AuthService::new(
            AuthConfig::default(),
            store.clone(),
            store.clone(),
            store.clone(),
            store.clone(),
            email.clone(),
        );

        let oauth_info = crate::oauth::OAuthUserInfo {
            provider_id: "google".to_string(),
            account_id: "google-123".to_string(),
            email: "oauth@example.com".to_string(),
            name: Some("OAuth User".to_string()),
            image: Some("https://example.com/avatar.jpg".to_string()),
        };

        let result = service
            .oauth_callback(
                oauth_info,
                OAuthTokens::default(),
                Some("127.0.0.1".to_string()),
                Some("test-agent".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(result.user.email, "oauth@example.com");
        assert_eq!(result.user.name, Some("OAuth User".to_string()));
        assert!(result.user.is_verified());
        assert!(result.user.password_hash.is_none());

        // Verify account was created
        let accounts = AccountStore::find_by_user_id(&store, result.user.id)
            .await
            .unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].provider_id, "google");
        assert_eq!(accounts[0].account_id, "google-123");
    }

    #[tokio::test]
    async fn oauth_callback_links_existing_user_by_email() {
        let store = MemoryStore::default();
        let email = TestEmailSender::default();
        let service = AuthService::new(
            AuthConfig::default(),
            store.clone(),
            store.clone(),
            store.clone(),
            store.clone(),
            email.clone(),
        );

        // Create existing user with password
        let existing_user = store
            .create_user(
                "existing@example.com",
                Some("Existing User"),
                Some("hash123"),
            )
            .await
            .unwrap();

        let oauth_info = crate::oauth::OAuthUserInfo {
            provider_id: "github".to_string(),
            account_id: "github-456".to_string(),
            email: "existing@example.com".to_string(),
            name: Some("GitHub User".to_string()),
            image: None,
        };

        let result = service
            .oauth_callback(
                oauth_info,
                OAuthTokens::default(),
                Some("127.0.0.1".to_string()),
                Some("test-agent".to_string()),
            )
            .await
            .unwrap();

        // Should link to existing user
        assert_eq!(result.user.id, existing_user.id);
        assert_eq!(result.user.email, "existing@example.com");
        assert!(result.user.is_verified());

        // Verify account was linked
        let accounts = AccountStore::find_by_user_id(&store, result.user.id)
            .await
            .unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].provider_id, "github");
        assert_eq!(accounts[0].account_id, "github-456");
    }

    #[tokio::test]
    async fn oauth_callback_logs_in_existing_account() {
        let store = MemoryStore::default();
        let email = TestEmailSender::default();
        let service = AuthService::new(
            AuthConfig::default(),
            store.clone(),
            store.clone(),
            store.clone(),
            store.clone(),
            email.clone(),
        );

        // Create user and account
        let user = store
            .create_user("oauth@example.com", Some("OAuth User"), None)
            .await
            .unwrap();
        store
            .create_account(crate::types::NewAccount {
                user_id: user.id,
                provider_id: "google".to_string(),
                account_id: "google-789".to_string(),
                access_token: None,
                refresh_token: None,
                access_token_expires_at: None,
                scope: None,
            })
            .await
            .unwrap();

        let oauth_info = crate::oauth::OAuthUserInfo {
            provider_id: "google".to_string(),
            account_id: "google-789".to_string(),
            email: "oauth@example.com".to_string(),
            name: Some("OAuth User".to_string()),
            image: None,
        };

        let result = service
            .oauth_callback(
                oauth_info,
                OAuthTokens::default(),
                Some("127.0.0.1".to_string()),
                Some("test-agent".to_string()),
            )
            .await
            .unwrap();

        // Should log in existing user
        assert_eq!(result.user.id, user.id);
        assert_eq!(result.user.email, "oauth@example.com");

        // Should not create duplicate account
        let accounts = AccountStore::find_by_user_id(&store, result.user.id)
            .await
            .unwrap();
        assert_eq!(accounts.len(), 1);
    }

    #[tokio::test]
    async fn oauth_callback_respects_linking_policy() {
        let store = MemoryStore::default();
        let email = TestEmailSender::default();
        let mut config = AuthConfig::default();
        config.oauth.allow_implicit_account_linking = false;

        let service = AuthService::new(
            config,
            store.clone(),
            store.clone(),
            store.clone(),
            store.clone(),
            email.clone(),
        );

        // Create existing user
        store
            .create_user(
                "existing@example.com",
                Some("Existing User"),
                Some("hash123"),
            )
            .await
            .unwrap();

        let oauth_info = crate::oauth::OAuthUserInfo {
            provider_id: "google".to_string(),
            account_id: "google-999".to_string(),
            email: "existing@example.com".to_string(),
            name: Some("OAuth User".to_string()),
            image: None,
        };

        // Should fail because linking is disabled
        let result = service
            .oauth_callback(
                oauth_info,
                OAuthTokens::default(),
                Some("127.0.0.1".to_string()),
                Some("test-agent".to_string()),
            )
            .await;

        assert!(result.is_err());
        match result {
            Err(AuthError::OAuth(msg)) => {
                assert!(msg.contains("Account linking by email is disabled"));
            }
            _ => panic!("Expected OAuth error"),
        }
    }
}
