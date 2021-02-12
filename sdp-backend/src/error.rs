use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ApiError {
    CommandNotInTimeIssuedBuffer,
    DatabaseConnFailed,
    HashingFailed,
    LoginFailedUserNotExist,
    LoginFailedPasswordIncorrect,
    CmdInstructionNotSupported,
    RobotInitializationFailed,
    SerializationError,
    AuthenticationFailed,
}

impl From<ApiError> for HttpResponse {
    fn from(error: ApiError) -> Self {
        // Get the json value for the error
        let error_json = match serde_json::to_string(&error) {
            Ok(j) => j,
            _ => return HttpResponse::InternalServerError().finish(),
        };

        // Give an appropiate error code for the error.
        match error {
            ApiError::CommandNotInTimeIssuedBuffer => HttpResponse::BadRequest().json(error_json),
            ApiError::DatabaseConnFailed => HttpResponse::InternalServerError().json(error_json),
            ApiError::HashingFailed => HttpResponse::InternalServerError().json(error_json),
            ApiError::LoginFailedUserNotExist => HttpResponse::Unauthorized().json(error_json),
            ApiError::LoginFailedPasswordIncorrect => HttpResponse::Unauthorized().json(error_json),
            ApiError::CmdInstructionNotSupported => HttpResponse::BadRequest().json(error_json),
            ApiError::RobotInitializationFailed => HttpResponse::BadRequest().json(error_json),
            ApiError::SerializationError => HttpResponse::InternalServerError().json(error_json),
            ApiError::AuthenticationFailed => HttpResponse::Unauthorized().json(error_json),
        }
    }
}
