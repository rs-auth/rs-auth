use axum_extra::extract::SignedCookieJar;
use axum_lib::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use rs_auth_core::AuthError;
use rs_auth_core::oauth::{client, github, google, providers};
use rs_auth_core::types::{NewVerification, PublicUser};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::cookie::set_session_cookie;
use crate::error::ApiError;
use crate::extract::ClientInfo;
use crate::state::AuthState;

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthCallbackResponse {
    pub user: PublicUser,
}

fn oauth_failure_response<U, S, V, A, E>(
    state: &AuthState<U, S, V, A, E>,
    error: AuthError,
) -> Result<Response, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    if let Some(error_url) = &state.config.oauth.error_redirect {
        Ok(Redirect::temporary(error_url).into_response())
    } else {
        Err(ApiError(error))
    }
}

// OAuth login handler - builds authorization URL and redirects
pub async fn oauth_login<U, S, V, A, E>(
    State(state): State<AuthState<U, S, V, A, E>>,
    Path(provider): Path<String>,
) -> Result<Response, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    // 1. Find provider in config
    let provider_entry = match state
        .config
        .oauth
        .providers
        .iter()
        .find(|p| p.provider_id == provider)
    {
        Some(entry) => entry,
        None => {
            return oauth_failure_response(
                &state,
                AuthError::OAuth(format!("Unknown provider: {}", provider)),
            );
        }
    };

    // 2. Build provider config
    let provider_config = match provider.as_str() {
        "google" => providers::google::google_provider(
            &provider_entry.client_id,
            &provider_entry.client_secret,
            &provider_entry.redirect_url,
            provider_entry.auth_url.as_deref(),
            provider_entry.token_url.as_deref(),
            provider_entry.userinfo_url.as_deref(),
        ),
        "github" => providers::github::github_provider(
            &provider_entry.client_id,
            &provider_entry.client_secret,
            &provider_entry.redirect_url,
            provider_entry.auth_url.as_deref(),
            provider_entry.token_url.as_deref(),
            provider_entry.userinfo_url.as_deref(),
        ),
        _ => {
            return oauth_failure_response(
                &state,
                AuthError::OAuth(format!("Unsupported provider: {}", provider)),
            );
        }
    };

    // 3. Build authorization URL
    let auth = match client::build_authorization(&provider_config) {
        Ok(auth) => auth,
        Err(error) => return oauth_failure_response(&state, error),
    };

    // 4. Store state and verifier in verifications table
    //
    // NOTE: OAuth state and PKCE verifiers are stored in the `verifications` table
    // rather than a dedicated `oauth_states` table. The identifier format is
    // `oauth-state:{csrf_token}` and the token_hash field stores the PKCE verifier.
    // This approach reuses existing infrastructure. If OAuth grows significantly,
    // consider migrating to a dedicated table for cleaner separation.
    //
    // We store the PKCE verifier in token_hash field (not hashed, we need it back)
    let identifier = format!("oauth-state:{}", auth.csrf_state);
    if let Err(error) = state
        .service
        .verifications
        .create_verification(NewVerification {
            identifier,
            token_hash: auth.pkce_verifier, // Store raw verifier
            expires_at: OffsetDateTime::now_utc() + state.config.verification_ttl,
        })
        .await
    {
        return oauth_failure_response(&state, error);
    }

    // 5. Redirect to authorization URL
    Ok(Redirect::temporary(&auth.authorize_url).into_response())
}

// OAuth callback handler - exchanges code for token and creates session
pub async fn oauth_callback<U, S, V, A, E>(
    State(state): State<AuthState<U, S, V, A, E>>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
    jar: SignedCookieJar,
    ClientInfo { ip, user_agent }: ClientInfo,
) -> Result<Response, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    // 1. Look up verification by state
    let identifier = format!("oauth-state:{}", query.state);
    let verification = match state
        .service
        .verifications
        .find_by_identifier(&identifier)
        .await
    {
        Ok(Some(verification)) => verification,
        Ok(None) => return oauth_failure_response(&state, AuthError::InvalidToken),
        Err(error) => return oauth_failure_response(&state, error),
    };

    // 2. Validate not expired
    if verification.expires_at < OffsetDateTime::now_utc() {
        if let Err(error) = state
            .service
            .verifications
            .delete_verification(verification.id)
            .await
        {
            return oauth_failure_response(&state, error);
        }
        return oauth_failure_response(&state, AuthError::InvalidToken);
    }

    // 3. Extract PKCE verifier from token_hash field
    let pkce_verifier = verification.token_hash.clone();

    // 4. Delete the verification
    if let Err(error) = state
        .service
        .verifications
        .delete_verification(verification.id)
        .await
    {
        return oauth_failure_response(&state, error);
    }

    // 5. Find provider in config
    let provider_entry = match state
        .config
        .oauth
        .providers
        .iter()
        .find(|p| p.provider_id == provider)
    {
        Some(entry) => entry,
        None => {
            return oauth_failure_response(
                &state,
                AuthError::OAuth(format!("Unknown provider: {}", provider)),
            );
        }
    };

    // 6. Build provider config
    let provider_config = match provider.as_str() {
        "google" => providers::google::google_provider(
            &provider_entry.client_id,
            &provider_entry.client_secret,
            &provider_entry.redirect_url,
            provider_entry.auth_url.as_deref(),
            provider_entry.token_url.as_deref(),
            provider_entry.userinfo_url.as_deref(),
        ),
        "github" => providers::github::github_provider(
            &provider_entry.client_id,
            &provider_entry.client_secret,
            &provider_entry.redirect_url,
            provider_entry.auth_url.as_deref(),
            provider_entry.token_url.as_deref(),
            provider_entry.userinfo_url.as_deref(),
        ),
        _ => {
            return oauth_failure_response(
                &state,
                AuthError::OAuth(format!("Unsupported provider: {}", provider)),
            );
        }
    };

    // 7. Exchange code for tokens
    let tokens = match client::exchange_code(&provider_config, &query.code, &pkce_verifier).await {
        Ok(tokens) => tokens,
        Err(error) => return oauth_failure_response(&state, error),
    };

    // 8. Fetch user info based on provider
    // Extract access_token for user info request
    let access_token = match tokens.access_token.as_ref() {
        Some(token) => token,
        None => {
            return oauth_failure_response(&state, AuthError::OAuth("No access token".to_string()));
        }
    };

    let user_info = match provider.as_str() {
        "google" => match google::fetch_user_info(&provider_config, access_token).await {
            Ok(user_info) => user_info,
            Err(error) => return oauth_failure_response(&state, error),
        },
        "github" => match github::fetch_user_info(&provider_config, access_token).await {
            Ok(user_info) => user_info,
            Err(error) => return oauth_failure_response(&state, error),
        },
        _ => {
            return oauth_failure_response(
                &state,
                AuthError::OAuth(format!("Unsupported provider: {}", provider)),
            );
        }
    };

    // 10. Call service oauth_callback
    let result = match state
        .service
        .oauth_callback(user_info, tokens, ip, user_agent)
        .await
    {
        Ok(result) => result,
        Err(error) => return oauth_failure_response(&state, error),
    };

    // 11. Set session cookie
    let jar = set_session_cookie(
        jar,
        &result.session_token,
        &state.config.cookie,
        state.config.session_ttl,
    );

    // 12. Return redirect or JSON based on config
    if let Some(success_url) = &state.config.oauth.success_redirect {
        Ok((jar, Redirect::temporary(success_url)).into_response())
    } else {
        Ok((
            jar,
            Json(OAuthCallbackResponse {
                user: result.user.into(),
            }),
        )
            .into_response())
    }
}
