use axum_extra::extract::SignedCookieJar;
use axum_lib::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use rs_auth_core::AuthError;
use rs_auth_core::error::OAuthError;
use rs_auth_core::oauth::{client, github, google, providers};
use rs_auth_core::types::{NewOAuthState, OAuthIntent, PublicUser};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{info, warn};

use crate::cookie::set_session_cookie;
use crate::error::ApiError;
use crate::extract::{ClientInfo, CurrentUser};
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

fn oauth_failure_response<U, S, V, A, O, E>(
    state: &AuthState<U, S, V, A, O, E>,
    provider: &str,
    phase: &str,
    error: AuthError,
) -> Result<Response, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let failure_class = match &error {
        AuthError::OAuth(oauth_error) => match oauth_error {
            OAuthError::ProviderNotFound { .. } => "provider_not_found",
            OAuthError::UnsupportedProvider { .. } => "unsupported_provider",
            OAuthError::Misconfigured { .. } => "misconfigured",
            OAuthError::InvalidState => "invalid_state",
            OAuthError::ExchangeFailed => "exchange_failed",
            OAuthError::UserInfoFailed => "userinfo_failed",
            OAuthError::UserInfoMalformed => "userinfo_malformed",
            OAuthError::MissingAccessToken => "missing_access_token",
            OAuthError::MissingEmail => "missing_email",
            OAuthError::LinkingDisabled => "linking_disabled",
            OAuthError::AccountNotFound => "account_not_found",
            OAuthError::LastAuthMethod => "last_auth_method",
            OAuthError::AccountAlreadyLinked => "account_already_linked",
            OAuthError::RefreshFailed => "refresh_failed",
            OAuthError::NoRefreshToken => "no_refresh_token",
        },
        AuthError::InvalidToken => "invalid_token",
        AuthError::Store(_) => "store_error",
        AuthError::Internal(_) => "internal_error",
        _ => "auth_error",
    };

    warn!(provider, phase, failure_class, "oauth flow failed");

    if let Some(error_url) = &state.config.oauth.error_redirect {
        Ok(Redirect::temporary(error_url).into_response())
    } else {
        Err(ApiError(error))
    }
}

fn resolve_provider_config(
    provider: &str,
    provider_entry: &rs_auth_core::config::OAuthProviderEntry,
) -> Result<rs_auth_core::oauth::OAuthProviderConfig, AuthError> {
    match provider {
        "google" => Ok(providers::google::google_provider(
            &provider_entry.client_id,
            &provider_entry.client_secret,
            &provider_entry.redirect_url,
            provider_entry.auth_url.as_deref(),
            provider_entry.token_url.as_deref(),
            provider_entry.userinfo_url.as_deref(),
        )),
        "github" => Ok(providers::github::github_provider(
            &provider_entry.client_id,
            &provider_entry.client_secret,
            &provider_entry.redirect_url,
            provider_entry.auth_url.as_deref(),
            provider_entry.token_url.as_deref(),
            provider_entry.userinfo_url.as_deref(),
        )),
        _ => Err(AuthError::OAuth(OAuthError::UnsupportedProvider {
            provider: provider.to_string(),
        })),
    }
}

fn find_provider_entry<'a>(
    config: &'a rs_auth_core::config::OAuthConfig,
    provider: &str,
) -> Result<&'a rs_auth_core::config::OAuthProviderEntry, AuthError> {
    config
        .providers
        .iter()
        .find(|p| p.provider_id == provider)
        .ok_or_else(|| {
            AuthError::OAuth(OAuthError::ProviderNotFound {
                provider: provider.to_string(),
            })
        })
}

pub async fn oauth_login<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    Path(provider): Path<String>,
) -> Result<Response, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    if let Err(error) = state.config.oauth.validate() {
        return oauth_failure_response(&state, &provider, "login.validate_config", error);
    }

    info!(
        provider = provider.as_str(),
        phase = "login.start",
        "starting oauth login"
    );

    let provider_entry = match find_provider_entry(&state.config.oauth, &provider) {
        Ok(entry) => entry,
        Err(error) => {
            return oauth_failure_response(&state, &provider, "login.provider_lookup", error);
        }
    };

    let provider_config = match resolve_provider_config(&provider, provider_entry) {
        Ok(config) => config,
        Err(error) => {
            return oauth_failure_response(&state, &provider, "login.provider_config", error);
        }
    };

    let auth = match client::build_authorization(&provider_config) {
        Ok(auth) => auth,
        Err(error) => {
            return oauth_failure_response(&state, &provider, "login.authorization", error);
        }
    };

    if let Err(error) = state
        .service
        .oauth_states
        .create_oauth_state(NewOAuthState {
            provider_id: provider.clone(),
            csrf_state: auth.csrf_state.clone(),
            pkce_verifier: auth.pkce_verifier,
            intent: OAuthIntent::Login,
            link_user_id: None,
            expires_at: OffsetDateTime::now_utc() + state.config.verification_ttl,
        })
        .await
    {
        return oauth_failure_response(&state, &provider, "login.persist_state", error);
    }

    info!(
        provider = provider.as_str(),
        phase = "login.redirect",
        "oauth login ready"
    );

    Ok(Redirect::temporary(&auth.authorize_url).into_response())
}

pub async fn oauth_link<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    Path(provider): Path<String>,
    CurrentUser { user, .. }: CurrentUser,
) -> Result<Response, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    if let Err(error) = state.config.oauth.validate() {
        return oauth_failure_response(&state, &provider, "link.validate_config", error);
    }

    info!(
        provider = provider.as_str(),
        user_id = user.id,
        phase = "link.start",
        "starting oauth account link"
    );

    let provider_entry = match find_provider_entry(&state.config.oauth, &provider) {
        Ok(entry) => entry,
        Err(error) => {
            return oauth_failure_response(&state, &provider, "link.provider_lookup", error);
        }
    };

    let provider_config = match resolve_provider_config(&provider, provider_entry) {
        Ok(config) => config,
        Err(error) => {
            return oauth_failure_response(&state, &provider, "link.provider_config", error);
        }
    };

    let auth = match client::build_authorization(&provider_config) {
        Ok(auth) => auth,
        Err(error) => {
            return oauth_failure_response(&state, &provider, "link.authorization", error);
        }
    };

    if let Err(error) = state
        .service
        .oauth_states
        .create_oauth_state(NewOAuthState {
            provider_id: provider.clone(),
            csrf_state: auth.csrf_state.clone(),
            pkce_verifier: auth.pkce_verifier,
            intent: OAuthIntent::Link,
            link_user_id: Some(user.id),
            expires_at: OffsetDateTime::now_utc() + state.config.verification_ttl,
        })
        .await
    {
        return oauth_failure_response(&state, &provider, "link.persist_state", error);
    }

    info!(
        provider = provider.as_str(),
        user_id = user.id,
        phase = "link.redirect",
        "oauth link ready"
    );

    Ok(Redirect::temporary(&auth.authorize_url).into_response())
}

pub async fn oauth_callback<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
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
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    if let Err(error) = state.config.oauth.validate() {
        return oauth_failure_response(&state, &provider, "callback.validate_config", error);
    }

    info!(
        provider = provider.as_str(),
        phase = "callback.start",
        "handling oauth callback"
    );

    let oauth_state = match state
        .service
        .oauth_states
        .find_by_csrf_state(&query.state)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return oauth_failure_response(
                &state,
                &provider,
                "callback.load_state",
                AuthError::OAuth(OAuthError::InvalidState),
            );
        }
        Err(error) => {
            return oauth_failure_response(&state, &provider, "callback.load_state", error);
        }
    };

    if oauth_state.expires_at < OffsetDateTime::now_utc() {
        let _ = state
            .service
            .oauth_states
            .delete_oauth_state(oauth_state.id)
            .await;
        return oauth_failure_response(
            &state,
            &provider,
            "callback.validate_state",
            AuthError::OAuth(OAuthError::InvalidState),
        );
    }

    let pkce_verifier = oauth_state.pkce_verifier.clone();
    let intent = oauth_state.intent;
    let link_user_id = oauth_state.link_user_id;

    if let Err(error) = state
        .service
        .oauth_states
        .delete_oauth_state(oauth_state.id)
        .await
    {
        return oauth_failure_response(&state, &provider, "callback.consume_state", error);
    }

    let provider_entry = match find_provider_entry(&state.config.oauth, &provider) {
        Ok(entry) => entry,
        Err(error) => {
            return oauth_failure_response(&state, &provider, "callback.provider_lookup", error);
        }
    };

    let provider_config = match resolve_provider_config(&provider, provider_entry) {
        Ok(config) => config,
        Err(error) => {
            return oauth_failure_response(&state, &provider, "callback.provider_config", error);
        }
    };

    let tokens = match client::exchange_code(&provider_config, &query.code, &pkce_verifier).await {
        Ok(tokens) => tokens,
        Err(error) => return oauth_failure_response(&state, &provider, "callback.exchange", error),
    };

    let access_token = match tokens.access_token.as_ref() {
        Some(token) => token,
        None => {
            return oauth_failure_response(
                &state,
                &provider,
                "callback.access_token",
                AuthError::OAuth(OAuthError::MissingAccessToken),
            );
        }
    };

    let user_info = match provider.as_str() {
        "google" => match google::fetch_user_info(&provider_config, access_token).await {
            Ok(user_info) => user_info,
            Err(error) => {
                return oauth_failure_response(&state, &provider, "callback.userinfo", error);
            }
        },
        "github" => match github::fetch_user_info(&provider_config, access_token).await {
            Ok(user_info) => user_info,
            Err(error) => {
                return oauth_failure_response(&state, &provider, "callback.userinfo", error);
            }
        },
        _ => {
            return oauth_failure_response(
                &state,
                &provider,
                "callback.userinfo",
                AuthError::OAuth(OAuthError::UnsupportedProvider {
                    provider: provider.clone(),
                }),
            );
        }
    };

    if intent == OAuthIntent::Link {
        let link_user_id =
            link_user_id.ok_or(ApiError(AuthError::OAuth(OAuthError::InvalidState)))?;

        match state
            .service
            .link_account(link_user_id, user_info, tokens)
            .await
        {
            Ok(_) => {}
            Err(error) => {
                return oauth_failure_response(&state, &provider, "callback.link", error);
            }
        };

        info!(
            provider = provider.as_str(),
            user_id = link_user_id,
            phase = "callback.link_success",
            "oauth account linked"
        );

        if let Some(success_url) = &state.config.oauth.success_redirect {
            return Ok(Redirect::temporary(success_url).into_response());
        }
        return Ok(Json(serde_json::json!({"linked": true})).into_response());
    }

    let result = match state
        .service
        .oauth_callback(user_info, tokens, ip, user_agent)
        .await
    {
        Ok(result) => result,
        Err(error) => return oauth_failure_response(&state, &provider, "callback.service", error),
    };

    info!(
        provider = provider.as_str(),
        phase = "callback.success",
        "oauth callback completed"
    );

    let jar = set_session_cookie(
        jar,
        &result.session_token,
        &state.config.cookie,
        state.config.session_ttl,
    );

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
