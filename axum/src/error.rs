use axum_lib::{Json, http::StatusCode, response::IntoResponse};
use rs_auth_core::AuthError;
use serde_json::json;

/// Wrapper for `AuthError` that implements `IntoResponse` for Axum handlers.
#[derive(Debug)]
pub struct ApiError(pub AuthError);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum_lib::response::Response {
        let (status, message) = match &self.0 {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid credentials"),
            AuthError::EmailTaken => (StatusCode::CONFLICT, "email already in use"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "user not found"),
            AuthError::SessionNotFound => {
                (StatusCode::UNAUTHORIZED, "session not found or expired")
            }
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "token invalid or expired"),
            AuthError::EmailNotVerified => (StatusCode::FORBIDDEN, "email not verified"),
            AuthError::WeakPassword(_) => (StatusCode::BAD_REQUEST, "password too weak"),
            AuthError::OAuth(_) => (StatusCode::BAD_REQUEST, "oauth error"),
            AuthError::Hash(_) | AuthError::Store(_) | AuthError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error")
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        Self(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_lib::http::StatusCode;

    #[test]
    fn invalid_credentials_maps_to_unauthorized() {
        let error = ApiError(AuthError::InvalidCredentials);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn email_taken_maps_to_conflict() {
        let error = ApiError(AuthError::EmailTaken);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn user_not_found_maps_to_not_found() {
        let error = ApiError(AuthError::UserNotFound);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn session_not_found_maps_to_unauthorized() {
        let error = ApiError(AuthError::SessionNotFound);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn invalid_token_maps_to_bad_request() {
        let error = ApiError(AuthError::InvalidToken);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn email_not_verified_maps_to_forbidden() {
        let error = ApiError(AuthError::EmailNotVerified);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn weak_password_maps_to_bad_request() {
        let error = ApiError(AuthError::WeakPassword(8));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn hash_error_maps_to_internal_server_error() {
        let error = ApiError(AuthError::Hash("hash error".to_string()));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn store_error_maps_to_internal_server_error() {
        let error = ApiError(AuthError::Store("store error".to_string()));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn internal_error_maps_to_internal_server_error() {
        let error = ApiError(AuthError::Internal("internal error".to_string()));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
