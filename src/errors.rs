use axum::{extract::rejection::FormRejection, http::StatusCode, response::IntoResponse};
use axum_macros::FromRequest;
use serde_json::json;

#[derive(FromRequest)]
#[from_request(via(axum::Form), rejection(FormError))]
pub struct Form<T>(pub T);

#[derive(Debug)]
pub struct FormError {
    code: StatusCode,
    message: String,
}

impl From<FormRejection> for FormError {
    fn from(rejection: FormRejection) -> Self {
        let code = match rejection {
            FormRejection::InvalidFormContentType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            FormRejection::BytesRejection(_) => StatusCode::BAD_REQUEST,
            FormRejection::FailedToDeserializeForm(_) => StatusCode::BAD_REQUEST,
            FormRejection::FailedToDeserializeFormBody(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Self {
            code,
            message: rejection.to_string(),
        }
    }
}

impl IntoResponse for FormError {
    fn into_response(self) -> axum::response::Response {
        let payload = json!({
            "message": self.message,
            "origin": "derive_from_request"
        });

        (self.code, axum::Json(payload)).into_response()
    }
}
