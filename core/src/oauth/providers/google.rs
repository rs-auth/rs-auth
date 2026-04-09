use crate::oauth::OAuthProviderConfig;

pub fn google_provider(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
    auth_url: Option<&str>,
    token_url: Option<&str>,
    userinfo_url: Option<&str>,
) -> OAuthProviderConfig {
    OAuthProviderConfig {
        provider_id: "google".to_string(),
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
        auth_url: auth_url
            .unwrap_or("https://accounts.google.com/o/oauth2/v2/auth")
            .to_string(),
        token_url: token_url
            .unwrap_or("https://oauth2.googleapis.com/token")
            .to_string(),
        userinfo_url: userinfo_url
            .unwrap_or("https://www.googleapis.com/oauth2/v3/userinfo")
            .to_string(),
        scopes: vec![
            "openid".to_string(),
            "email".to_string(),
            "profile".to_string(),
        ],
        redirect_url: redirect_url.to_string(),
    }
}
