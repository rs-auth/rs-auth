use axum_lib::{
    Json,
    extract::{Path, State},
};
use rs_auth_core::types::PublicAccount;
use serde::Serialize;

use crate::error::ApiError;
use crate::extract::CurrentUser;
use crate::state::AuthState;

#[derive(Debug, Serialize)]
pub struct ListAccountsResponse {
    pub accounts: Vec<PublicAccount>,
}

#[derive(Debug, Serialize)]
pub struct UnlinkAccountResponse {
    pub success: bool,
}

pub async fn list_accounts<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    CurrentUser { user, .. }: CurrentUser,
) -> Result<Json<ListAccountsResponse>, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    let accounts = state.service.list_accounts(user.id).await?;
    Ok(Json(ListAccountsResponse { accounts }))
}

pub async fn unlink_account<U, S, V, A, O, E>(
    State(state): State<AuthState<U, S, V, A, O, E>>,
    CurrentUser { user, .. }: CurrentUser,
    Path(account_id): Path<i64>,
) -> Result<Json<UnlinkAccountResponse>, ApiError>
where
    U: rs_auth_core::store::UserStore + Send + Sync + 'static,
    S: rs_auth_core::store::SessionStore + Send + Sync + 'static,
    V: rs_auth_core::store::VerificationStore + Send + Sync + 'static,
    A: rs_auth_core::store::AccountStore + Send + Sync + 'static,
    O: rs_auth_core::store::OAuthStateStore + Send + Sync + 'static,
    E: rs_auth_core::email::EmailSender + Send + Sync + 'static,
{
    state.service.unlink_account(user.id, account_id).await?;
    Ok(Json(UnlinkAccountResponse { success: true }))
}
