use crate::error::{AuthError, OAuthError};
use crate::oauth::{OAuthProviderConfig, OAuthUserInfo};
use serde::Deserialize;

#[derive(Deserialize)]
struct GitHubUserResponse {
    id: i64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
    email: Option<String>,
}

#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

pub async fn fetch_user_info(
    config: &OAuthProviderConfig,
    access_token: &str,
) -> Result<OAuthUserInfo, AuthError> {
    let client = reqwest::Client::new();
    let user_response = client
        .get(&config.userinfo_url)
        .bearer_auth(access_token)
        .header("User-Agent", "rs-auth")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| AuthError::OAuth(OAuthError::UserInfoFailed))?;

    if !user_response.status().is_success() {
        return Err(AuthError::OAuth(OAuthError::UserInfoFailed));
    }

    let user: GitHubUserResponse = user_response
        .json()
        .await
        .map_err(|_| AuthError::OAuth(OAuthError::UserInfoMalformed))?;

    let email = if let Some(email) = user.email {
        email
    } else {
        let emails_response = client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "rs-auth")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|_| AuthError::OAuth(OAuthError::UserInfoFailed))?;

        if !emails_response.status().is_success() {
            return Err(AuthError::OAuth(OAuthError::UserInfoFailed));
        }

        let emails: Vec<GitHubEmail> = emails_response
            .json()
            .await
            .map_err(|_| AuthError::OAuth(OAuthError::UserInfoMalformed))?;
        emails
            .into_iter()
            .find(|e| e.primary && e.verified)
            .map(|e| e.email)
            .ok_or(AuthError::OAuth(OAuthError::MissingEmail))?
    };

    Ok(OAuthUserInfo {
        provider_id: "github".to_string(),
        account_id: user.id.to_string(),
        email,
        name: user.name.or(Some(user.login)),
        image: user.avatar_url,
    })
}
