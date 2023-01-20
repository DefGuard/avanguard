use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
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

#[derive(Debug, Error)]
pub enum Web3Error {
    #[error("hex decoding error")]
    Decode,
    #[error("invalid message")]
    InvalidMessage,
    #[error("invalid recovery id")]
    InvalidRecoveryId,
    #[error("error parsing signature")]
    ParseSignature,
    #[error("recovery error")]
    Recovery,
    #[error("error verifying address")]
    VerifyAddress,
}

#[derive(Debug, Error, PartialEq)]
pub enum HexError {
    #[error("Invalid character {0}")]
    InvalidCharacter(u8),
    #[error("Invalid string length {0}")]
    InvalidStringLength(usize),
}
