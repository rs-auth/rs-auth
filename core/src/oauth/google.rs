use crate::error::{AuthError, OAuthError};
use crate::oauth::{OAuthProviderConfig, OAuthUserInfo};
use serde::Deserialize;

#[derive(Deserialize)]
struct GoogleUserResponse {
    sub: String,
    email: String,
    name: Option<String>,
    picture: Option<String>,
}

pub async fn fetch_user_info(
    config: &OAuthProviderConfig,
    access_token: &str,
) -> Result<OAuthUserInfo, AuthError> {
    let client = reqwest::Client::new();
    let response = client
        .get(&config.userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|_| AuthError::OAuth(OAuthError::UserInfoFailed))?;

    if !response.status().is_success() {
        return Err(AuthError::OAuth(OAuthError::UserInfoFailed));
    }

    let resp: GoogleUserResponse = response
        .json()
        .await
        .map_err(|_| AuthError::OAuth(OAuthError::UserInfoMalformed))?;
    Ok(OAuthUserInfo {
        provider_id: "google".to_string(),
        account_id: resp.sub,
        email: resp.email,
        name: resp.name,
        image: resp.picture,
    })
}
