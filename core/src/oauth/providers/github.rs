use crate::oauth::OAuthProviderConfig;

pub fn github_provider(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
    auth_url: Option<&str>,
    token_url: Option<&str>,
    userinfo_url: Option<&str>,
) -> OAuthProviderConfig {
    OAuthProviderConfig {
        provider_id: "github".to_string(),
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
        auth_url: auth_url
            .unwrap_or("https://github.com/login/oauth/authorize")
            .to_string(),
        token_url: token_url
            .unwrap_or("https://github.com/login/oauth/access_token")
            .to_string(),
        userinfo_url: userinfo_url
            .unwrap_or("https://api.github.com/user")
            .to_string(),
        scopes: vec!["user:email".to_string()],
        redirect_url: redirect_url.to_string(),
    }
}
