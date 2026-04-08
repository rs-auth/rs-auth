use axum_lib::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("unauthorized")]
    Unauthorized,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum_lib::response::Response {
        (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
    }
}
