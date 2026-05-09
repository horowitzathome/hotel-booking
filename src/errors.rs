use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("{0}")]
    UnprocessableEntity(String),

    #[error("validation failed")]
    ValidationError(Vec<String>),

    #[error("internal server error")]
    Internal(#[source] anyhow::Error),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if let Self::Internal(e) = self {
            tracing::error!(error = ?e, "internal server error");
            return HttpResponse::InternalServerError()
                .json(json!({"error": "internal server error"}));
        }

        let body = match self {
            Self::ValidationError(details) => {
                json!({"error": self.to_string(), "details": details})
            }
            _ => json!({"error": self.to_string()}),
        };

        HttpResponse::build(self.status_code()).json(body)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::RowNotFound => AppError::NotFound("resource not found".to_string()),
            sqlx::Error::Database(db) => match db.code().as_deref() {
                // Postgres: unique_violation
                Some("23505") => AppError::Conflict(db.message().to_string()),
                // Postgres: foreign_key_violation
                Some("23503") => AppError::Conflict(db.message().to_string()),
                _ => AppError::Internal(anyhow::anyhow!("{e}")),
            },
            _ => AppError::Internal(anyhow::anyhow!("{e}")),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e)
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut details = Vec::new();
        for (field, kind) in errors.errors() {
            match kind {
                validator::ValidationErrorsKind::Field(errs) => {
                    for err in errs {
                        details.push(format!("{field}: {}", err.code));
                    }
                }
                validator::ValidationErrorsKind::Struct(inner) => {
                    for d in AppError::from(*inner.clone()).validation_details() {
                        details.push(format!("{field}.{d}"));
                    }
                }
                validator::ValidationErrorsKind::List(map) => {
                    for (idx, inner) in map {
                        for d in AppError::from(*inner.clone()).validation_details() {
                            details.push(format!("{field}[{idx}].{d}"));
                        }
                    }
                }
            }
        }
        AppError::ValidationError(details)
    }
}

impl AppError {
    fn validation_details(self) -> Vec<String> {
        match self {
            AppError::ValidationError(d) => d,
            _ => Vec::new(),
        }
    }
}
