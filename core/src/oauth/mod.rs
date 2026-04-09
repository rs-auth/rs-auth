pub mod client;
pub mod github;
pub mod google;
pub mod providers;

use serde::{Deserialize, Serialize};

/// Information about a user obtained from an OAuth provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider_id: String,
    pub account_id: String,
    pub email: String,
    pub name: Option<String>,
    pub image: Option<String>,
}

/// Configuration for an OAuth provider.
#[derive(Debug, Clone)]
pub struct OAuthProviderConfig {
    pub provider_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub scopes: Vec<String>,
    pub redirect_url: String,
}

/// OAuth tokens returned from provider.
#[derive(Debug, Clone, Default)]
pub struct OAuthTokens {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<time::Duration>,
    pub scope: Option<String>,
}
