use crate::command::{Command, Instruction};
use crate::error::ApiError;
use crate::user::User;

use actix_web::{delete, get, post, web, web::Data, HttpRequest, HttpResponse};
use chrono::{serde::ts_seconds, Utc};
use serde::Deserialize;
use sqlx::postgres::PgPool;
use std::str::FromStr;

#[derive(Deserialize, Debug)]
pub struct CommandRequest {
    #[serde(with = "ts_seconds")]
    time_issued: chrono::DateTime<Utc>,
    // time_instruction: Vec<chrono::DateTime<Utc>>,
    time_instruction: Vec<u64>,
    instruction: Instruction,
}

#[post("/command")]
pub async fn create_command(
    conn: Data<PgPool>,
    user: User,
    cmd: web::Json<CommandRequest>,
) -> HttpResponse {
    let time_instruction: Vec<chrono::DateTime<Utc>> =
        match convert_time(cmd.time_instruction.clone()) {
            Ok(t) => t,
            Err(e) => return e.into(),
        };

    Command::batch_new(
        &conn,
        &user.robot_serial_number,
        cmd.time_issued,
        time_instruction,
        // cmd.time_instruction.clone(),
        &cmd.instruction,
    )
    .await
    .map_or_else(|e| e.into(), |cmd| HttpResponse::Ok().json(cmd))
}

#[get("/command/{value}")]
pub async fn get_command(conn: Data<PgPool>, user: User, req: HttpRequest) -> HttpResponse {
    let command_id = parse_req::<i64>(&req, "value").await.ok();
    let robot_serial_number = parse_req::<String>(&req, "value").await.ok();

    match (command_id, robot_serial_number) {
        // Get a specific command
        (Some(id), _) => {
            match Command::get_by_id(&conn, id).await {
                // If there is an error getting the command return it
                Err(e) => e.into(),
                // If the logged in users rsn is the same as the one for the command, return it
                Ok(c) if user.robot_serial_number == c.robot_serial_number => {
                    HttpResponse::Ok().json(c)
                }
                // If the user is not allowed to view the command, return an error
                Ok(_) => ApiError::AuthenticationFailed.into(),
            }
        }

        // Get all of the commands for this robot_serial_number
        (_, Some(rsn)) => match Command::get_all_by_robot_serial_number(&conn, &rsn).await {
            _ if rsn != user.robot_serial_number => ApiError::AuthenticationFailed.into(),
            Ok(cs) => HttpResponse::Ok().json(cs),
            Err(e) => e.into(),
        },

        (None, None) => ApiError::SerializationError.into(),
    }
}

#[delete("/command/{command_id}")]
pub async fn cancel_command(conn: Data<PgPool>, user: User, req: HttpRequest) -> HttpResponse {
    let command_id = match parse_req::<i64>(&req, "command_id").await {
        Ok(c) => c,
        Err(e) => return e.into(),
    };

    match Command::get_by_id(&conn, command_id).await {
        // If there is an error getting the command return it
        Err(e) => e.into(),
        // If the logged in users rsn is the same as the one for the command, return it
        Ok(c) if user.robot_serial_number == c.robot_serial_number => c
            .cancel(&conn)
            .await
            .map_or_else(|e| e.into(), |cmd| HttpResponse::Ok().json(cmd)),
        // If the user is not allowed to view the command, return an error
        Ok(_) => ApiError::AuthenticationFailed.into(),
    }
}

#[derive(Deserialize)]
struct Time {
    #[serde(with = "ts_seconds")]
    time: chrono::DateTime<Utc>,
}

// TODO: Improve this, want to use normal serialize or an into trait
fn convert_time(times: Vec<u64>) -> Result<Vec<chrono::DateTime<Utc>>, ApiError> {
    let mut time_instructions = Vec::new();
    for t in times {
        let json_time = format!("{{\"time\": {}}}", t.to_string());
        let formatted_time: Time =
            serde_json::from_str(&json_time).map_err(|_| ApiError::SerializationError)?;
        time_instructions.push(formatted_time.time);
    }

    Ok(time_instructions)
}

// Given a a HttpRequest get the parmaters passed into the path
// They can be of any type implementing the FromStr trait
async fn parse_req<T>(req: &HttpRequest, param: &str) -> Result<T, ApiError>
where
    T: FromStr,
{
    let req_string = req
        .match_info()
        .get(param)
        .ok_or(ApiError::SerializationError)?;

    req_string
        .parse::<T>()
        .map_err(|_| ApiError::SerializationError)
}
