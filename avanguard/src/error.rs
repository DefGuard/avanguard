use actix_web::{ResponseError, HttpResponse, http::{header::ContentType, StatusCode}};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("sqlx error")]
    Sqlx(#[from] sqlx::Error),
}

impl ApiError {
    pub fn code(&self) -> &str {
        match self {
            Self::Sqlx(_) => "DB",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Sqlx(e) => e.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ErrorInfo {
    error: String,
    message: String,
}

impl From<&ApiError> for ErrorInfo {
    fn from(api_error: &ApiError) -> Self {
        Self {
            error: api_error.code().into(),
            message: api_error.message(),
        }
    }
}

impl ResponseError for ApiError {
    /// Return error as JSON.
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(ErrorInfo::from(self))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
