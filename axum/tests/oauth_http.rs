use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum_lib::body::Body;
use axum_lib::extract::State;
use axum_lib::http::{Request, StatusCode};
use axum_lib::{
    Json, Router,
    routing::{get, post},
};
use http_body_util::BodyExt;
use rs_auth_axum::{AuthState, auth_router};
use rs_auth_core::AuthService;
use rs_auth_core::config::{AuthConfig, OAuthConfig, OAuthProviderEntry};
use rs_auth_core::email::EmailSender;
use rs_auth_core::error::AuthError;
use rs_auth_core::store::{
    AccountStore, OAuthStateStore, SessionStore, UserStore, VerificationStore,
};
use rs_auth_core::types::{
    Account, NewAccount, NewOAuthState, NewSession, NewVerification, OAuthState, Session, User,
    Verification,
};
use serde_json::json;
use time::OffsetDateTime;
use tower::ServiceExt;

// ============================================================================
// In-memory test stores (same as auth_http.rs)
// ============================================================================

#[derive(Default)]
struct MemoryState {
    next_user_id: i64,
    next_session_id: i64,
    next_verification_id: i64,
    next_account_id: i64,
    next_oauth_state_id: i64,
    users: HashMap<i64, User>,
    sessions: HashMap<i64, Session>,
    verifications: HashMap<i64, Verification>,
    accounts: HashMap<i64, Account>,
    oauth_states: HashMap<i64, OAuthState>,
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

    async fn update_password(&self, user_id: i64, password_hash: &str) -> Result<(), AuthError> {
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
            .find(|account| account.provider_id == provider_id && account.account_id == account_id)
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

#[async_trait]
impl OAuthStateStore for MemoryStore {
    async fn create_oauth_state(&self, new_state: NewOAuthState) -> Result<OAuthState, AuthError> {
        let mut state = self.inner.lock().unwrap();
        state.next_oauth_state_id += 1;
        let now = OffsetDateTime::now_utc();
        let oauth_state = OAuthState {
            id: state.next_oauth_state_id,
            provider_id: new_state.provider_id,
            csrf_state: new_state.csrf_state,
            pkce_verifier: new_state.pkce_verifier,
            expires_at: new_state.expires_at,
            created_at: now,
        };
        state
            .oauth_states
            .insert(oauth_state.id, oauth_state.clone());
        Ok(oauth_state)
    }

    async fn find_by_csrf_state(&self, csrf_state: &str) -> Result<Option<OAuthState>, AuthError> {
        let state = self.inner.lock().unwrap();
        Ok(state
            .oauth_states
            .values()
            .find(|s| s.csrf_state == csrf_state)
            .cloned())
    }

    async fn delete_oauth_state(&self, id: i64) -> Result<(), AuthError> {
        self.inner.lock().unwrap().oauth_states.remove(&id);
        Ok(())
    }

    async fn delete_expired_oauth_states(&self) -> Result<u64, AuthError> {
        let now = OffsetDateTime::now_utc();
        let mut state = self.inner.lock().unwrap();
        let before = state.oauth_states.len();
        state.oauth_states.retain(|_, s| s.expires_at >= now);
        Ok((before - state.oauth_states.len()) as u64)
    }
}

// ============================================================================
// Test email sender
// ============================================================================

#[derive(Clone, Default)]
struct TestEmailSender {
    verification_tokens: Arc<Mutex<Vec<String>>>,
    reset_tokens: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl EmailSender for TestEmailSender {
    async fn send_verification_email(&self, _user: &User, token: &str) -> Result<(), AuthError> {
        self.verification_tokens
            .lock()
            .unwrap()
            .push(token.to_string());
        Ok(())
    }

    async fn send_password_reset_email(&self, _user: &User, token: &str) -> Result<(), AuthError> {
        self.reset_tokens.lock().unwrap().push(token.to_string());
        Ok(())
    }
}

// ============================================================================
// Test app builder with OAuth configuration
// ============================================================================

fn test_oauth_app(store: MemoryStore, email: TestEmailSender) -> axum_lib::Router {
    let config = AuthConfig {
        secret: "long-enough-secret-for-cookie-signing-at-least-64-bytes-long-here".to_string(),
        oauth: OAuthConfig {
            providers: vec![
                OAuthProviderEntry {
                    provider_id: "google".to_string(),
                    client_id: "test-google-client-id".to_string(),
                    client_secret: "test-google-client-secret".to_string(),
                    redirect_url: "http://localhost:3000/auth/callback/google".to_string(),
                    auth_url: None,
                    token_url: None,
                    userinfo_url: None,
                },
                OAuthProviderEntry {
                    provider_id: "github".to_string(),
                    client_id: "test-github-client-id".to_string(),
                    client_secret: "test-github-client-secret".to_string(),
                    redirect_url: "http://localhost:3000/auth/callback/github".to_string(),
                    auth_url: None,
                    token_url: None,
                    userinfo_url: None,
                },
            ],
            allow_implicit_account_linking: true,
            success_redirect: None,
            error_redirect: None,
        },
        ..Default::default()
    };
    let service = AuthService::new(
        config,
        store.clone(),
        store.clone(),
        store.clone(),
        store.clone(),
        store,
        email,
    );
    let state = AuthState::new(service);
    auth_router(state.clone()).with_state(state)
}

fn test_oauth_app_with_google_override(
    store: MemoryStore,
    email: TestEmailSender,
    base_url: &str,
    success_redirect: Option<String>,
) -> axum_lib::Router {
    let config = AuthConfig {
        secret: "long-enough-secret-for-cookie-signing-at-least-64-bytes-long-here".to_string(),
        oauth: OAuthConfig {
            providers: vec![OAuthProviderEntry {
                provider_id: "google".to_string(),
                client_id: "test-google-client-id".to_string(),
                client_secret: "test-google-client-secret".to_string(),
                redirect_url: "http://localhost:3000/auth/callback/google".to_string(),
                auth_url: Some(format!("{base_url}/authorize")),
                token_url: Some(format!("{base_url}/token")),
                userinfo_url: Some(format!("{base_url}/userinfo")),
            }],
            allow_implicit_account_linking: true,
            success_redirect,
            error_redirect: None,
        },
        ..Default::default()
    };
    let service = AuthService::new(
        config,
        store.clone(),
        store.clone(),
        store.clone(),
        store.clone(),
        store,
        email,
    );
    let state = AuthState::new(service);
    auth_router(state.clone()).with_state(state)
}

fn test_oauth_app_with_google_override_and_error_redirect(
    store: MemoryStore,
    email: TestEmailSender,
    base_url: &str,
    error_redirect: Option<String>,
) -> axum_lib::Router {
    let config = AuthConfig {
        secret: "long-enough-secret-for-cookie-signing-at-least-64-bytes-long-here".to_string(),
        oauth: OAuthConfig {
            providers: vec![OAuthProviderEntry {
                provider_id: "google".to_string(),
                client_id: "test-google-client-id".to_string(),
                client_secret: "test-google-client-secret".to_string(),
                redirect_url: "http://localhost:3000/auth/callback/google".to_string(),
                auth_url: Some(format!("{base_url}/authorize")),
                token_url: Some(format!("{base_url}/token")),
                userinfo_url: Some(format!("{base_url}/userinfo")),
            }],
            allow_implicit_account_linking: true,
            success_redirect: None,
            error_redirect,
        },
        ..Default::default()
    };
    let service = AuthService::new(
        config,
        store.clone(),
        store.clone(),
        store.clone(),
        store.clone(),
        store,
        email,
    );
    let state = AuthState::new(service);
    auth_router(state.clone()).with_state(state)
}

async fn spawn_mock_google_provider() -> String {
    spawn_mock_google_provider_with(
        StatusCode::OK,
        json!({
            "access_token": "mock-access-token",
            "refresh_token": "mock-refresh-token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "openid email profile"
        }),
        StatusCode::OK,
        json!({
            "sub": "google-user-123",
            "email": "oauth@example.com",
            "name": "OAuth User",
            "picture": "https://example.com/avatar.png"
        }),
    )
    .await
}

async fn spawn_mock_google_provider_with(
    token_status: StatusCode,
    token_payload: serde_json::Value,
    userinfo_status: StatusCode,
    userinfo_payload: serde_json::Value,
) -> String {
    async fn authorize() -> &'static str {
        "ok"
    }

    async fn token(
        State((status, payload)): State<(StatusCode, serde_json::Value)>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        (status, Json(payload))
    }

    async fn userinfo(
        State((status, payload)): State<(StatusCode, serde_json::Value)>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        (status, Json(payload))
    }

    let app = Router::new()
        .route("/authorize", get(authorize))
        .route(
            "/token",
            post(token).with_state((token_status, token_payload)),
        )
        .route(
            "/userinfo",
            get(userinfo).with_state((userinfo_status, userinfo_payload)),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum_lib::serve(listener, app).await.unwrap();
    });
    format!("http://{}", addr)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn send_request(
    app: axum_lib::Router,
    request: Request<Body>,
) -> (StatusCode, String, Vec<(String, String)>) {
    // Call the router using oneshot
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Extract headers
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
        .collect();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    (status, body_str, headers)
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn oauth_login_google_returns_redirect() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_oauth_app(store, email);

    let request = Request::builder()
        .method("GET")
        .uri("/login/google")
        .body(Body::empty())
        .unwrap();

    let (status, _body, headers) = send_request(app, request).await;

    // Should return a redirect status (303 See Other or 307 Temporary Redirect)
    assert!(
        status == StatusCode::SEE_OTHER || status == StatusCode::TEMPORARY_REDIRECT,
        "Expected redirect status, got: {}",
        status
    );

    // Should have a Location header pointing to Google's OAuth
    let location = headers
        .iter()
        .find(|(name, _)| name == "location")
        .map(|(_, value)| value);

    assert!(location.is_some(), "Location header should be present");
    let location = location.unwrap();
    assert!(
        location.contains("accounts.google.com"),
        "Location should point to Google OAuth, got: {}",
        location
    );
}

#[tokio::test]
async fn oauth_login_github_returns_redirect() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_oauth_app(store, email);

    let request = Request::builder()
        .method("GET")
        .uri("/login/github")
        .body(Body::empty())
        .unwrap();

    let (status, _body, headers) = send_request(app, request).await;

    // Should return a redirect status
    assert!(
        status == StatusCode::SEE_OTHER || status == StatusCode::TEMPORARY_REDIRECT,
        "Expected redirect status, got: {}",
        status
    );

    // Should have a Location header pointing to GitHub's OAuth
    let location = headers
        .iter()
        .find(|(name, _)| name == "location")
        .map(|(_, value)| value);

    assert!(location.is_some(), "Location header should be present");
    let location = location.unwrap();
    assert!(
        location.contains("github.com/login/oauth/authorize"),
        "Location should point to GitHub OAuth, got: {}",
        location
    );
}

#[tokio::test]
async fn oauth_login_unknown_provider_returns_error() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_oauth_app(store, email);

    let request = Request::builder()
        .method("GET")
        .uri("/login/unknown")
        .body(Body::empty())
        .unwrap();

    let (status, _body, _headers) = send_request(app, request).await;

    // Should return an error status (400 Bad Request or 404 Not Found)
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND,
        "Expected error status for unknown provider, got: {}",
        status
    );
}

#[tokio::test]
async fn oauth_callback_invalid_state_returns_error() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_oauth_app(store, email);

    let request = Request::builder()
        .method("GET")
        .uri("/callback/google?code=test-code&state=invalid-state")
        .body(Body::empty())
        .unwrap();

    let (status, _body, _headers) = send_request(app, request).await;

    // Should return an error status (400 Bad Request or 401 Unauthorized)
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNAUTHORIZED,
        "Expected error status for invalid state, got: {}",
        status
    );
}

#[tokio::test]
async fn oauth_callback_missing_code_returns_error() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_oauth_app(store, email);

    let request = Request::builder()
        .method("GET")
        .uri("/callback/google?state=test-state")
        .body(Body::empty())
        .unwrap();

    let (status, _body, _headers) = send_request(app, request).await;

    // Should return an error status (400 Bad Request or 422 Unprocessable Entity)
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected error status for missing code, got: {}",
        status
    );
}

#[tokio::test]
async fn oauth_callback_expired_state_returns_error() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let store_clone = store.clone();

    // Pre-seed an expired OAuth state
    store
        .create_oauth_state(NewOAuthState {
            provider_id: "google".to_string(),
            csrf_state: "test-state-123".to_string(),
            pkce_verifier: "test-pkce-verifier".to_string(),
            expires_at: OffsetDateTime::now_utc() - time::Duration::hours(1),
        })
        .await
        .unwrap();

    let app = test_oauth_app(store, email);

    let request = Request::builder()
        .method("GET")
        .uri("/callback/google?code=test-code&state=test-state-123")
        .body(Body::empty())
        .unwrap();

    let (status, _body, _headers) = send_request(app, request).await;

    // Should return an error status (400 Bad Request or 401 Unauthorized)
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNAUTHORIZED,
        "Expected error status for expired state, got: {}",
        status
    );

    assert!(
        store_clone
            .find_by_csrf_state("test-state-123")
            .await
            .unwrap()
            .is_none(),
        "expired oauth state should be deleted after callback"
    );
}

#[tokio::test]
async fn oauth_callback_valid_state_fails_at_exchange() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // Pre-seed a valid OAuth state
    store
        .create_oauth_state(NewOAuthState {
            provider_id: "google".to_string(),
            csrf_state: "valid-state".to_string(),
            pkce_verifier: "test-pkce-verifier".to_string(),
            expires_at: OffsetDateTime::now_utc() + time::Duration::hours(1),
        })
        .await
        .unwrap();

    let app = test_oauth_app(store, email);

    let request = Request::builder()
        .method("GET")
        .uri("/callback/google?code=fake-code&state=valid-state")
        .body(Body::empty())
        .unwrap();

    let (status, _body, _headers) = send_request(app, request).await;

    // Should return an error status - the state lookup and validation works,
    // but the code exchange with Google will fail since it's a fake code
    assert!(
        status == StatusCode::BAD_REQUEST
            || status == StatusCode::UNAUTHORIZED
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected error status for failed code exchange, got: {}",
        status
    );
}

#[tokio::test]
async fn oauth_login_stores_state_in_oauth_states() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // Keep a clone of the store to inspect it after the request
    let store_clone = store.clone();

    let app = test_oauth_app(store, email);

    let request = Request::builder()
        .method("GET")
        .uri("/login/google")
        .body(Body::empty())
        .unwrap();

    let (status, _body, headers) = send_request(app, request).await;

    // Should return a redirect status
    assert!(
        status == StatusCode::SEE_OTHER || status == StatusCode::TEMPORARY_REDIRECT,
        "Expected redirect status, got: {}",
        status
    );

    // Extract the state parameter from the Location header
    let location = headers
        .iter()
        .find(|(name, _)| name == "location")
        .map(|(_, value)| value)
        .expect("Location header should be present");

    // Parse the state parameter from the URL
    let url = url::Url::parse(location).expect("Location should be a valid URL");
    let state_param = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("State parameter should be present in authorization URL");

    // Check that an OAuth state record was created with the correct state
    let oauth_state = store_clone
        .find_by_csrf_state(&state_param)
        .await
        .unwrap()
        .expect("OAuth state should exist for login state");

    assert_eq!(oauth_state.provider_id, "google");
    assert_eq!(oauth_state.csrf_state, state_param);
    assert!(!oauth_state.pkce_verifier.is_empty());
    // Verify it's not expired
    assert!(oauth_state.expires_at > OffsetDateTime::now_utc());
}

#[tokio::test]
async fn oauth_callback_success_returns_json_and_sets_cookie() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let provider_url = spawn_mock_google_provider().await;

    store
        .create_oauth_state(NewOAuthState {
            provider_id: "google".to_string(),
            csrf_state: "success-state".to_string(),
            pkce_verifier: "pkce-verifier".to_string(),
            expires_at: OffsetDateTime::now_utc() + time::Duration::hours(1),
        })
        .await
        .unwrap();

    let app = test_oauth_app_with_google_override(store, email, &provider_url, None);
    let request = Request::builder()
        .method("GET")
        .uri("/callback/google?code=realistic-code&state=success-state")
        .body(Body::empty())
        .unwrap();

    let (status, body, headers) = send_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("oauth@example.com"));
    let set_cookie = headers.iter().find(|(name, _)| name == "set-cookie");
    assert!(
        set_cookie.is_some(),
        "successful callback should set a session cookie"
    );
}

#[tokio::test]
async fn oauth_callback_replay_fails_after_successful_callback() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let provider_url = spawn_mock_google_provider().await;

    store
        .create_oauth_state(NewOAuthState {
            provider_id: "google".to_string(),
            csrf_state: "replay-state".to_string(),
            pkce_verifier: "pkce-verifier".to_string(),
            expires_at: OffsetDateTime::now_utc() + time::Duration::hours(1),
        })
        .await
        .unwrap();

    let app = test_oauth_app_with_google_override(store, email, &provider_url, None);
    let request = Request::builder()
        .method("GET")
        .uri("/callback/google?code=realistic-code&state=replay-state")
        .body(Body::empty())
        .unwrap();

    let (status, _body, _headers) = send_request(app.clone(), request).await;
    assert_eq!(status, StatusCode::OK);

    let replay_request = Request::builder()
        .method("GET")
        .uri("/callback/google?code=realistic-code&state=replay-state")
        .body(Body::empty())
        .unwrap();

    let (replay_status, _body, _headers) = send_request(app, replay_request).await;
    assert_eq!(replay_status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn oauth_callback_success_redirects_when_configured() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let provider_url = spawn_mock_google_provider().await;

    store
        .create_oauth_state(NewOAuthState {
            provider_id: "google".to_string(),
            csrf_state: "success-redirect-state".to_string(),
            pkce_verifier: "pkce-verifier".to_string(),
            expires_at: OffsetDateTime::now_utc() + time::Duration::hours(1),
        })
        .await
        .unwrap();

    let app = test_oauth_app_with_google_override(
        store,
        email,
        &provider_url,
        Some("/dashboard".to_string()),
    );
    let request = Request::builder()
        .method("GET")
        .uri("/callback/google?code=realistic-code&state=success-redirect-state")
        .body(Body::empty())
        .unwrap();

    let (status, _body, headers) = send_request(app, request).await;

    assert!(
        status == StatusCode::SEE_OTHER || status == StatusCode::TEMPORARY_REDIRECT,
        "expected redirect status, got {status}"
    );
    let location = headers
        .iter()
        .find(|(name, _)| name == "location")
        .map(|(_, value)| value.as_str());
    assert_eq!(location, Some("/dashboard"));
    let set_cookie = headers.iter().find(|(name, _)| name == "set-cookie");
    assert!(
        set_cookie.is_some(),
        "successful callback should set a session cookie"
    );
}

#[tokio::test]
async fn oauth_callback_error_redirects_when_configured() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let provider_url = spawn_mock_google_provider().await;

    let app = test_oauth_app_with_google_override_and_error_redirect(
        store,
        email,
        &provider_url,
        Some("/login?error=oauth".to_string()),
    );
    let request = Request::builder()
        .method("GET")
        .uri("/callback/google?code=realistic-code&state=missing-state")
        .body(Body::empty())
        .unwrap();

    let (status, _body, headers) = send_request(app, request).await;

    assert!(
        status == StatusCode::SEE_OTHER || status == StatusCode::TEMPORARY_REDIRECT,
        "expected redirect status, got {status}"
    );
    let location = headers
        .iter()
        .find(|(name, _)| name == "location")
        .map(|(_, value)| value.as_str());
    assert_eq!(location, Some("/login?error=oauth"));
}

#[tokio::test]
async fn oauth_callback_malformed_userinfo_returns_error() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let provider_url = spawn_mock_google_provider_with(
        StatusCode::OK,
        json!({
            "access_token": "mock-access-token",
            "refresh_token": "mock-refresh-token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "scope": "openid email profile"
        }),
        StatusCode::OK,
        json!({
            "not_sub": "missing expected google fields"
        }),
    )
    .await;

    store
        .create_oauth_state(NewOAuthState {
            provider_id: "google".to_string(),
            csrf_state: "malformed-userinfo-state".to_string(),
            pkce_verifier: "pkce-verifier".to_string(),
            expires_at: OffsetDateTime::now_utc() + time::Duration::hours(1),
        })
        .await
        .unwrap();

    let app = test_oauth_app_with_google_override(store, email, &provider_url, None);
    let request = Request::builder()
        .method("GET")
        .uri("/callback/google?code=realistic-code&state=malformed-userinfo-state")
        .body(Body::empty())
        .unwrap();

    let (status, _body, _headers) = send_request(app, request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn oauth_login_duplicate_provider_config_returns_server_error() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let config = AuthConfig {
        secret: "long-enough-secret-for-cookie-signing-at-least-64-bytes-long-here".to_string(),
        oauth: OAuthConfig {
            providers: vec![
                OAuthProviderEntry {
                    provider_id: "google".to_string(),
                    client_id: "a".to_string(),
                    client_secret: "b".to_string(),
                    redirect_url: "http://localhost:3000/auth/callback/google".to_string(),
                    auth_url: None,
                    token_url: None,
                    userinfo_url: None,
                },
                OAuthProviderEntry {
                    provider_id: "google".to_string(),
                    client_id: "c".to_string(),
                    client_secret: "d".to_string(),
                    redirect_url: "http://localhost:3000/auth/callback/google-2".to_string(),
                    auth_url: None,
                    token_url: None,
                    userinfo_url: None,
                },
            ],
            allow_implicit_account_linking: true,
            success_redirect: None,
            error_redirect: None,
        },
        ..Default::default()
    };
    let service = AuthService::new(
        config,
        store.clone(),
        store.clone(),
        store.clone(),
        store.clone(),
        store,
        email,
    );
    let state = AuthState::new(service);
    let app = auth_router(state.clone()).with_state(state);

    let request = Request::builder()
        .method("GET")
        .uri("/login/google")
        .body(Body::empty())
        .unwrap();

    let (status, _body, _headers) = send_request(app, request).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}
