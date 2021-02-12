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
