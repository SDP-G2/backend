#[cfg(test)]
mod tests {
    use crate::command::Command;
    use crate::command::{
        AbortReason,
        AbortReason::{LowBattery, Obstacle, Saftey},
        CleaningPattern,
        CleaningPattern::{Circular, ZigZag},
        Instruction,
        Instruction::{Abort, Idle, Task},
    };
    use crate::poll::Poll;
    use crate::user::User;

    use chrono::{serde::ts_seconds, Utc};
    use sqlx::postgres::PgPool;
    use std::env;

    async fn delete_data(conn: &PgPool, robot_serial_number: &str) {
        sqlx::query!(
            r#"
DELETE FROM Commands
WHERE robot_serial_number=$1
        "#,
            robot_serial_number
        )
        .execute(conn)
        .await
        .ok();

        sqlx::query!(
            r#"
DELETE FROM Users
WHERE robot_serial_number=$1
        "#,
            robot_serial_number
        )
        .execute(conn)
        .await
        .ok();

        sqlx::query!(
            r#"
DELETE FROM Robot
WHERE robot_serial_number=$1
        "#,
            robot_serial_number
        )
        .execute(conn)
        .await
        .ok();
    }

    async fn db_connect() -> PgPool {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL to be set");
        let database_pool = PgPool::connect(&database_url)
            .await
            .expect("to get database pool");

        database_pool
    }

    async fn setup_tests<'a>() -> (&'a str, &'a str, &'a str, PgPool) {
        let conn = &db_connect().await;

        let user = "test_user";
        let password = "password";
        let rsn = "test_serial";

        delete_data(conn, rsn).await;

        // Create a new user and associated robot
        User::new(conn, user, password, rsn).await.unwrap();

        (user, password, rsn, conn.clone())
    }

    #[actix_rt::test]
    async fn set_idle_poll_idle() {
        let (_user, _password, rsn, conn) = setup_tests().await;

        let time_now = chrono::Utc::now();

        let prev_cmd = Command::new(&conn, rsn, time_now, time_now, &Instruction::Idle)
            .await
            .unwrap();

        let poll = Poll {
            robot_serial_number: rsn.to_string(),
            instruction: Idle,
            battery_level: 90,
        };

        let result = Poll::poll(&conn, &poll).await.unwrap();
        let cmd = Command::current(&conn, rsn).await.unwrap();
        let prev_cmd_updated = Command::get_by_id(&conn, prev_cmd.command_id)
            .await
            .unwrap();

        // Check the right response
        assert_eq!(Idle, result.instruction);

        // Check the result was entered into the db
        assert_eq!(result.clone(), cmd);

        // Check the old command is not complete yet
        assert_eq!(false, prev_cmd_updated.completed);

        // Check the new command is the same as the old one
        assert_eq!(result, prev_cmd_updated)
    }

    #[actix_rt::test]
    async fn set_idle_abort_low_battery() {
        let (_user, _password, rsn, conn) = setup_tests().await;

        let time_now = chrono::Utc::now();

        let prev_cmd = Command::new(
            &conn,
            rsn,
            time_now,
            time_now,
            &Instruction::Task(CleaningPattern::ZigZag),
        )
        .await
        .unwrap();

        let poll = Poll {
            robot_serial_number: rsn.to_string(),
            instruction: Idle,
            battery_level: 10,
        };

        let result = Poll::poll(&conn, &poll).await.unwrap();
        let cmd = Command::current(&conn, rsn).await.unwrap();
        let prev_cmd_updated = Command::get_by_id(&conn, prev_cmd.command_id)
            .await
            .unwrap();

        // Check the right response
        assert_eq!(Abort(AbortReason::LowBattery), result.instruction);

        // Check the result was entered into the db
        assert_eq!(result.clone(), cmd);

        // Check the old commands was marked as complete
        assert_eq!(true, prev_cmd_updated.completed);
    }

    #[actix_rt::test]
    async fn set_task_idle() {
        let (_user, _password, rsn, conn) = setup_tests().await;

        let time_now = chrono::Utc::now();

        let prev_cmd = Command::new(
            &conn,
            rsn,
            time_now,
            time_now,
            &Instruction::Task(CleaningPattern::ZigZag),
        )
        .await
        .unwrap();

        let poll = Poll {
            robot_serial_number: rsn.to_string(),
            instruction: Idle,
            battery_level: 90,
        };

        let result = Poll::poll(&conn, &poll).await.unwrap();
        let cmd = Command::current(&conn, rsn).await.unwrap();
        let prev_cmd_updated = Command::get_by_id(&conn, prev_cmd.command_id)
            .await
            .unwrap();

        // Check the right response
        assert_eq!(Idle, result.instruction);

        // Check the result was entered into the db
        assert_eq!(result.clone(), cmd);

        // Check the old commands was marked as complete
        assert_eq!(true, prev_cmd_updated.completed);
    }

    #[actix_rt::test]
    async fn set_abort_low_battery_idle() {
        let (_user, _password, rsn, conn) = setup_tests().await;

        let time_now = chrono::Utc::now();

        let prev_cmd = Command::new(
            &conn,
            rsn,
            time_now,
            time_now,
            &Instruction::Abort(AbortReason::LowBattery),
        )
        .await
        .unwrap();

        let poll = Poll {
            robot_serial_number: rsn.to_string(),
            instruction: Idle,
            battery_level: 90,
        };

        let result = Poll::poll(&conn, &poll).await.unwrap();
        let cmd = Command::current(&conn, rsn).await.unwrap();
        let prev_cmd_updated = Command::get_by_id(&conn, prev_cmd.command_id)
            .await
            .unwrap();

        // Check the right response
        assert_eq!(Idle, result.instruction);

        // Check the result was entered into the db
        assert_eq!(result.clone(), cmd);

        // Check the old commands was marked as complete
        assert_eq!(true, prev_cmd_updated.completed);
    }

    #[actix_rt::test]
    async fn set_task_pending_task() {
        let (_user, _password, rsn, conn) = setup_tests().await;

        let time_now = chrono::Utc::now();

        let cmd_task1 = Command::new(
            &conn,
            rsn,
            time_now,
            time_now,
            &Instruction::Task(CleaningPattern::Circular),
        )
        .await
        .unwrap();

        let poll = Poll {
            robot_serial_number: rsn.to_string(),
            instruction: Idle,
            battery_level: 90,
        };

        let result = Poll::poll(&conn, &poll).await.unwrap();
        let cmd = Command::current(&conn, rsn).await.unwrap();
        let cmd_updated = Command::get_by_id(&conn, cmd_task1.command_id)
            .await
            .unwrap();

        // Check the right response
        assert_eq!(Idle, result.instruction);

        // Check the result was entered into the db
        assert_eq!(result.clone(), cmd);

        // Check the old commands was marked as complete
        assert_eq!(true, cmd_updated.completed);

        let time_now = chrono::Utc::now();

        let cmd_task2 = Command::new(
            &conn,
            rsn,
            time_now,
            time_now,
            &Instruction::Task(CleaningPattern::ZigZag),
        )
        .await
        .unwrap();

        let poll = Poll {
            robot_serial_number: rsn.to_string(),
            instruction: Idle,
            battery_level: 90,
        };

        let result = Poll::poll(&conn, &poll).await.unwrap();
        let cmd = Command::current(&conn, rsn).await.unwrap();
        let cmd_updated = Command::get_by_id(&conn, cmd_task2.command_id)
            .await
            .unwrap();

        // Check the right response
        assert_eq!(Task(CleaningPattern::ZigZag), result.instruction);

        // Check the result was entered into the db
        assert_eq!(result.clone(), cmd);

        // Check the old commands was marked as complete
        assert_eq!(false, cmd_updated.completed);
    }

    #[actix_rt::test]
    async fn set_abort_task_task() {
        let (_user, _password, rsn, conn) = setup_tests().await;

        let time_now = chrono::Utc::now();

        let prev_cmd = Command::new(
            &conn,
            rsn,
            time_now,
            time_now,
            &Instruction::Task(CleaningPattern::Circular),
        )
        .await
        .unwrap();

        let poll = Poll {
            robot_serial_number: rsn.to_string(),
            instruction: Abort(AbortReason::Obstacle),
            battery_level: 90,
        };

        let result = Poll::poll(&conn, &poll).await.unwrap();
        let cmd = Command::current(&conn, rsn).await.unwrap();
        let prev_cmd_updated = Command::get_by_id(&conn, prev_cmd.command_id)
            .await
            .unwrap();

        // Check the right response
        assert_eq!(Abort(AbortReason::Obstacle), result.instruction);

        // Check the result was entered into the db
        assert_eq!(result.clone(), cmd);

        // Check the old commands was marked as complete
        assert_eq!(true, prev_cmd_updated.completed);

        let time_now = chrono::Utc::now();

        let prev_cmd = Command::new(
            &conn,
            rsn,
            time_now,
            time_now,
            &Instruction::Task(CleaningPattern::ZigZag),
        )
        .await
        .unwrap();

        let poll = Poll {
            robot_serial_number: rsn.to_string(),
            instruction: Idle,
            battery_level: 90,
        };

        let result = Poll::poll(&conn, &poll).await.unwrap();
        let cmd = Command::current(&conn, rsn).await.unwrap();
        let prev_cmd_updated = Command::get_by_id(&conn, prev_cmd.command_id)
            .await
            .unwrap();

        // Check the right response
        assert_eq!(Task(CleaningPattern::ZigZag), result.instruction);

        // Check the result was entered into the db
        assert_eq!(result.clone(), cmd);

        // Check the old commands was marked as complete
        assert_eq!(false, prev_cmd_updated.completed);
    }
}
