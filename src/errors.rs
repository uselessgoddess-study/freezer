use actix_web::{http::StatusCode, ResponseError};
use anyhow::anyhow;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(anyhow::Error),
    #[error("not found: {0}")]
    LogicError(anyhow::Error),
    #[error("not found: {0}")]
    ActixWeb(actix_web::Error),
    #[error("")]
    TooManyRequests { actual: u64, permitted: u64 },

    #[error(transparent)]
    Internal(#[from] anyhow::Error),

    #[error(transparent)]
    Mongo(#[from] mongodb::error::Error),
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::ActixWeb(err) => err.as_response_error().status_code(),
            Error::LogicError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Error::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Internal(error.into())
    }
}

impl From<actix_web::Error> for Error {
    fn from(error: actix_web::Error) -> Self {
        Self::Internal(error.into())
    }
}

pub macro not_found($($tt:tt)*) {
    Err(Error::NotFound(anyhow!($($tt)*)))
}
