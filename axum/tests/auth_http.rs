use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum_lib::body::Body;
use axum_lib::http::{Request, StatusCode};
use http_body_util::BodyExt;
use rs_auth_axum::{AuthState, auth_router};
use rs_auth_core::AuthService;
use rs_auth_core::config::AuthConfig;
use rs_auth_core::email::EmailSender;
use rs_auth_core::error::AuthError;
use rs_auth_core::store::{AccountStore, SessionStore, UserStore, VerificationStore};
use rs_auth_core::types::{
    Account, NewAccount, NewSession, NewVerification, Session, User, Verification,
};
use time::OffsetDateTime;
use tower::ServiceExt;

// ============================================================================
// In-memory test stores (cloned from core/src/service.rs tests)
// ============================================================================

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

// ============================================================================
// Test email sender
// ============================================================================

#[derive(Clone, Default)]
struct TestEmailSender {
    verification_tokens: Arc<Mutex<Vec<String>>>,
    reset_tokens: Arc<Mutex<Vec<String>>>,
}

impl TestEmailSender {
    #[allow(dead_code)]
    fn get_last_verification_token(&self) -> Option<String> {
        self.verification_tokens.lock().unwrap().last().cloned()
    }

    fn get_last_reset_token(&self) -> Option<String> {
        self.reset_tokens.lock().unwrap().last().cloned()
    }
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
// Test app builder
// ============================================================================

fn test_app(store: MemoryStore, email: TestEmailSender) -> axum_lib::Router {
    let config = AuthConfig {
        secret: "test-secret-that-is-long-enough-for-cookie-signing-key-padding".to_string(),
        ..Default::default()
    };
    let service = AuthService::new(
        config,
        store.clone(),
        store.clone(),
        store.clone(),
        store,
        email,
    );
    let state = AuthState::new(service);
    auth_router(state.clone()).with_state(state)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn send_request(app: axum_lib::Router, request: Request<Body>) -> (StatusCode, String) {
    // Call the router using oneshot
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    (status, body_str)
}

async fn send_request_with_headers(
    app: axum_lib::Router,
    request: Request<Body>,
) -> (StatusCode, axum_lib::http::HeaderMap, String) {
    // Call the router using oneshot
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    (status, headers, body_str)
}

fn json_request(method: &str, uri: &str, json: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn signup_returns_201_with_user_json() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_app(store, email);

    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret","name":"Test User"}"#,
    );

    let (status, body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body.contains("test@example.com"));
    assert!(body.contains("Test User"));
}

#[tokio::test]
async fn signup_with_short_password_returns_400() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_app(store, email);

    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"short"}"#,
    );

    let (status, _body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn signup_with_duplicate_email_returns_409() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // First signup
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, _) = send_request(app, request).await;
    assert_eq!(status, StatusCode::CREATED);

    // Second signup with same email
    let app = test_app(store, email);
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, _body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn login_with_valid_credentials_returns_200() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // First signup
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, _) = send_request(app, request).await;
    assert_eq!(status, StatusCode::CREATED);

    // Then login
    let app = test_app(store, email);
    let request = json_request(
        "POST",
        "/login",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("test@example.com"));
}

#[tokio::test]
async fn login_with_wrong_password_returns_401() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // First signup
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, _) = send_request(app, request).await;
    assert_eq!(status, StatusCode::CREATED);

    // Then login with wrong password
    let app = test_app(store, email);
    let request = json_request(
        "POST",
        "/login",
        r#"{"email":"test@example.com","password":"wrongpassword"}"#,
    );
    let (status, _body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_with_nonexistent_email_returns_401() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_app(store, email);

    let request = json_request(
        "POST",
        "/login",
        r#"{"email":"nonexistent@example.com","password":"supersecret"}"#,
    );
    let (status, _body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_without_cookie_returns_401() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_app(store, email);

    let request = Request::builder()
        .method("POST")
        .uri("/logout")
        .body(Body::empty())
        .unwrap();

    let (status, _body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn forgot_password_always_returns_200() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_app(store, email);

    // Request password reset for non-existent email
    let request = json_request("POST", "/forgot", r#"{"email":"nonexistent@example.com"}"#);
    let (status, body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("\"ok\":true"));
}

#[tokio::test]
async fn reset_password_with_valid_token_returns_200() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // First signup
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, _) = send_request(app, request).await;
    assert_eq!(status, StatusCode::CREATED);

    // Request password reset
    let app = test_app(store.clone(), email.clone());
    let request = json_request("POST", "/forgot", r#"{"email":"test@example.com"}"#);
    let (status, _) = send_request(app, request).await;
    assert_eq!(status, StatusCode::OK);

    // Get the reset token
    let reset_token = email
        .get_last_reset_token()
        .expect("reset token should exist");

    // Reset password with valid token
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/reset",
        &format!(r#"{{"token":"{}","password":"newpassword"}}"#, reset_token),
    );
    let (status, body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("\"ok\":true"));

    // Verify we can login with new password
    let app = test_app(store, email);
    let request = json_request(
        "POST",
        "/login",
        r#"{"email":"test@example.com","password":"newpassword"}"#,
    );
    let (status, _) = send_request(app, request).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_session_returns_200_with_valid_cookie() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // Sign up a user
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, headers, _body) = send_request_with_headers(app, request).await;
    assert_eq!(status, StatusCode::CREATED);

    // Extract the set-cookie header
    let cookie_header = headers
        .get("set-cookie")
        .expect("set-cookie header should be present")
        .to_str()
        .unwrap();

    // Send GET /session with the cookie
    let app = test_app(store, email);
    let request = Request::builder()
        .method("GET")
        .uri("/session")
        .header("cookie", cookie_header)
        .body(Body::empty())
        .unwrap();
    let (status, body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("test@example.com"));
}

#[tokio::test]
async fn get_session_returns_401_without_cookie() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_app(store, email);

    // Send GET /session with no cookie
    let request = Request::builder()
        .method("GET")
        .uri("/session")
        .body(Body::empty())
        .unwrap();
    let (status, _body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_sessions_returns_active_sessions() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // Sign up a user
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, headers, _body) = send_request_with_headers(app, request).await;
    assert_eq!(status, StatusCode::CREATED);

    // Extract the set-cookie header
    let cookie_header = headers
        .get("set-cookie")
        .expect("set-cookie header should be present")
        .to_str()
        .unwrap();

    // Send GET /sessions with the cookie
    let app = test_app(store, email);
    let request = Request::builder()
        .method("GET")
        .uri("/sessions")
        .header("cookie", cookie_header)
        .body(Body::empty())
        .unwrap();
    let (status, body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("sessions"));
}

#[tokio::test]
async fn verify_email_returns_200_for_valid_token() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // Sign up a user
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, _) = send_request(app, request).await;
    assert_eq!(status, StatusCode::CREATED);

    // Get the verification token from the TestEmailSender
    let verification_token = email
        .get_last_verification_token()
        .expect("verification token should exist");

    // Send GET /verify/{token}
    let app = test_app(store, email);
    let request = Request::builder()
        .method("GET")
        .uri(format!("/verify/{}", verification_token))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("test@example.com"));
}

#[tokio::test]
async fn signup_sets_session_cookie() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();
    let app = test_app(store, email);

    // Sign up a user
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, headers, _body) = send_request_with_headers(app, request).await;

    assert_eq!(status, StatusCode::CREATED);

    // Assert the response has a set-cookie header containing "rs_auth_session"
    let cookie_header = headers
        .get("set-cookie")
        .expect("set-cookie header should be present")
        .to_str()
        .unwrap();
    assert!(cookie_header.contains("rs_auth_session"));
}

#[tokio::test]
async fn login_sets_session_cookie() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // First signup
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, _) = send_request(app, request).await;
    assert_eq!(status, StatusCode::CREATED);

    // Then login
    let app = test_app(store, email);
    let request = json_request(
        "POST",
        "/login",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, headers, _body) = send_request_with_headers(app, request).await;

    assert_eq!(status, StatusCode::OK);

    // Assert the login response has a set-cookie header
    let cookie_header = headers
        .get("set-cookie")
        .expect("set-cookie header should be present")
        .to_str()
        .unwrap();
    assert!(cookie_header.contains("rs_auth_session"));
}

#[tokio::test]
async fn logout_removes_session_cookie() {
    let store = MemoryStore::default();
    let email = TestEmailSender::default();

    // Sign up a user
    let app = test_app(store.clone(), email.clone());
    let request = json_request(
        "POST",
        "/signup",
        r#"{"email":"test@example.com","password":"supersecret"}"#,
    );
    let (status, headers, _body) = send_request_with_headers(app, request).await;
    assert_eq!(status, StatusCode::CREATED);

    // Extract the set-cookie header
    let cookie_header = headers
        .get("set-cookie")
        .expect("set-cookie header should be present")
        .to_str()
        .unwrap();

    // POST /logout with that cookie
    let app = test_app(store, email);
    let request = Request::builder()
        .method("POST")
        .uri("/logout")
        .header("cookie", cookie_header)
        .body(Body::empty())
        .unwrap();
    let (status, headers, _body) = send_request_with_headers(app, request).await;

    assert_eq!(status, StatusCode::OK);

    // Assert the logout response has a set-cookie header that removes/expires the cookie
    let cookie_header = headers
        .get("set-cookie")
        .expect("set-cookie header should be present")
        .to_str()
        .unwrap();
    assert!(cookie_header.contains("rs_auth_session"));
    // When a cookie is removed, it typically has Max-Age=0 or an expired date
    assert!(
        cookie_header.contains("Max-Age=0") || cookie_header.contains("max-age=0"),
        "Cookie should be expired/removed"
    );
}
