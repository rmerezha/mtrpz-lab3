use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::error::Error::Anyhow;

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum Error {
    /// Return `400 Bad Request`
    #[error("bad request")]
    BadRequest,

    /// Return `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Return `413 Content Too Large`
    #[error("Content Too Large")]
    PayloadTooLarge,

    /// Return `500 Internal Server Error` on a `tokio_postgres::Error`.
    #[error("an error occurred with the database")]
    Postgres,

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

impl From<tokio_pg_mapper::Error> for Error {
    fn from(value: tokio_pg_mapper::Error) -> Self {
        Anyhow(anyhow::anyhow!(value))
    }
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Postgres | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest => {
                tracing::error!("Bad request");
            }

            Self::Unauthorized => {
                tracing::error!("Unauthorized error: {:?}", self.to_string());
                return (
                    self.status_code(),
                    self.to_string(),
                )
                    .into_response();
            }

            Self::PayloadTooLarge => {
                tracing::warn!("Payload too large");
                return (
                    self.status_code(),
                    "Payload too large".to_string(),
                )
                    .into_response();
            }

            Self::Postgres => {
                tracing::error!("Postgres connection error");
            }

            Self::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
            }

            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}
