use crate::api_types::ApiErrorBody;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}
impl ApiError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
    pub fn forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: "You can only change your own stories.".into(),
        }
    }
    pub fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: "Story not found.".into(),
        }
    }
    pub fn conflict() -> Self {
        Self { status: StatusCode::CONFLICT, message: "This story changed in another tab. Your text is still here. Copy it before reloading the latest version.".into() }
    }
    pub fn busy() -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            message: "Too many attempts. Please wait a minute and try again.".into(),
        }
    }
    pub fn database(_: sqlx::Error) -> Self {
        Self::internal("The database operation could not be completed. Please try again.")
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}
