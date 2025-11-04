use std::fmt::Display;

use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError, body::BoxBody, dev::ServiceResponse, http::{StatusCode, header::{self, ContentType, TryIntoHeaderValue}}, middleware::ErrorHandlerResponse};
use serde::Serialize;
use sqlx::{Pool, Postgres};

pub struct AppState {
    pub db_pool: Pool<Postgres>,
}


#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum AppError {
    BadRequest { message: String },
    Internal,
    NotFound,
    DuplicateUsername,
    UserNotFound,
    SRPNotStarted,
    InvalidCredential,
}

#[derive(Serialize)]
struct AppErrorResponse {
    error: AppError,
}

impl Display for AppError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidCredential => StatusCode::UNAUTHORIZED,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let mut res = HttpResponse::new(self.status_code());

        res.headers_mut().insert(
            header::CONTENT_TYPE,
            ContentType::json().try_into_value().unwrap(),
        );

        res.set_body(BoxBody::new(
            serde_json::to_string(&AppErrorResponse {
                error: self.clone(),
            })
            .unwrap(),
        ))
    }
}

pub struct AppResult<T>(pub Result<T, AppError>);

impl<T> AppResult<T> {
    pub fn ok(value: T) -> Self {
        AppResult(Ok(value))
    }

    pub fn err(error: AppError) -> Self {
        AppResult(Err(error))
    }
}

impl<T> Responder for AppResult<T>
where
    T: Serialize,
{
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        match &self.0 {
            Ok(value) => {
                let body = serde_json::to_string(value).unwrap();

                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(body)
            }
            Err(e) => e.error_response(),
        }
    }
}


pub fn global_error_handler<B>(
    res: ServiceResponse<B>,
) -> actix_web::Result<ErrorHandlerResponse<B>> {
    let status = res.status();

    let (req, res) = res.into_parts();
    let err = match status {
        StatusCode::NOT_FOUND => AppError::NotFound,
        StatusCode::INTERNAL_SERVER_ERROR => AppError::Internal,
        _ => {
            return Ok(ErrorHandlerResponse::Response(
                ServiceResponse::new(req, res).map_into_left_body(),
            ));
        }
    };

    let res =
        res.set_body(serde_json::to_string(&AppErrorResponse { error: err.clone() }).unwrap());

    let res = ServiceResponse::new(req, res)
        .map_into_boxed_body()
        .map_into_right_body();

    Ok(ErrorHandlerResponse::Response(res))
}