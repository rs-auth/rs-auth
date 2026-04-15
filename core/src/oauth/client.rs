use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
};

use crate::error::{AuthError, OAuthError};
use crate::oauth::{OAuthProviderConfig, OAuthTokens};

// Type alias for a fully configured OAuth client
type ConfiguredClient = BasicClient<
    EndpointSet,    // HasAuthUrl
    EndpointNotSet, // HasDeviceAuthUrl
    EndpointNotSet, // HasIntrospectionUrl
    EndpointNotSet, // HasRevocationUrl
    EndpointSet,    // HasTokenUrl
>;

/// Result of building an OAuth authorization URL.
pub struct OAuthAuthorization {
    pub authorize_url: String,
    pub csrf_state: String,
    pub pkce_verifier: String,
}

/// Build an OAuth authorization URL with PKCE.
pub fn build_authorization(config: &OAuthProviderConfig) -> Result<OAuthAuthorization, AuthError> {
    let client = build_client(config)?;
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut auth_request = client.authorize_url(CsrfToken::new_random);
    auth_request = auth_request.set_pkce_challenge(pkce_challenge);
    for scope in &config.scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.clone()));
    }
    let (auth_url, csrf_token) = auth_request.url();

    Ok(OAuthAuthorization {
        authorize_url: auth_url.to_string(),
        csrf_state: csrf_token.secret().clone(),
        pkce_verifier: pkce_verifier.secret().clone(),
    })
}

/// Exchange an authorization code for access and refresh tokens.
pub async fn exchange_code(
    config: &OAuthProviderConfig,
    code: &str,
    pkce_verifier: &str,
) -> Result<OAuthTokens, AuthError> {
    let client = build_client(config)?;

    // Create a reqwest client that doesn't follow redirects (to prevent SSRF)
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| AuthError::OAuth(OAuthError::ExchangeFailed))?;

    let token_response = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
        .request_async(&http_client)
        .await
        .map_err(|_| AuthError::OAuth(OAuthError::ExchangeFailed))?;

    let access_token = token_response.access_token().secret().clone();
    let refresh_token = token_response.refresh_token().map(|t| t.secret().clone());

    // Extract expires_in and convert from std::time::Duration to time::Duration
    let expires_in = token_response
        .expires_in()
        .and_then(|d| time::Duration::try_from(d).ok());

    // Extract scopes and join them into a single string
    let scope = token_response.scopes().map(|scopes| {
        scopes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    });

    Ok(OAuthTokens {
        access_token: Some(access_token),
        refresh_token,
        expires_in,
        scope,
    })
}

/// Refresh an access token using a stored refresh token.
pub async fn refresh_access_token(
    config: &OAuthProviderConfig,
    refresh_token_str: &str,
) -> Result<OAuthTokens, AuthError> {
    let client = build_client(config)?;

    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| AuthError::OAuth(OAuthError::RefreshFailed))?;

    let token_response = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token_str.to_string()))
        .request_async(&http_client)
        .await
        .map_err(|_| AuthError::OAuth(OAuthError::RefreshFailed))?;

    let access_token = token_response.access_token().secret().clone();
    let refresh_token = token_response.refresh_token().map(|t| t.secret().clone());

    let expires_in = token_response
        .expires_in()
        .and_then(|d| time::Duration::try_from(d).ok());

    let scope = token_response.scopes().map(|scopes| {
        scopes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    });

    Ok(OAuthTokens {
        access_token: Some(access_token),
        refresh_token,
        expires_in,
        scope,
    })
}

fn build_client(config: &OAuthProviderConfig) -> Result<ConfiguredClient, AuthError> {
    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_client_secret(ClientSecret::new(config.client_secret.clone()))
        .set_auth_uri(AuthUrl::new(config.auth_url.clone()).map_err(|e| {
            AuthError::OAuth(OAuthError::Misconfigured {
                message: format!("invalid auth_url: {e}"),
            })
        })?)
        .set_token_uri(TokenUrl::new(config.token_url.clone()).map_err(|e| {
            AuthError::OAuth(OAuthError::Misconfigured {
                message: format!("invalid token_url: {e}"),
            })
        })?)
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone()).map_err(|e| {
            AuthError::OAuth(OAuthError::Misconfigured {
                message: format!("invalid redirect_url: {e}"),
            })
        })?);
    Ok(client)
}
