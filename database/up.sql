CREATE TABLE IF NOT EXISTS Robot (
       robot_serial_number VARCHAR PRIMARY KEY,
       battery_level BIGINT NOT NULL DEFAULT 75,
       assigned BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS Users (
       user_id BIGSERIAL PRIMARY KEY,
       user_name VARCHAR NOT NULL UNIQUE,
       password_hash VARCHAR NOT NULL,
       robot_serial_number VARCHAR NOT NULL UNIQUE REFERENCES Robot(robot_serial_number)
);

CREATE TABLE IF NOT EXISTS Commands (
       command_id BIGSERIAL PRIMARY KEY,
       robot_serial_number VARCHAR NOT NULL REFERENCES Robot(robot_serial_number),
       time_issued timestamptz NOT NULL,
       time_instruction timestamptz NOT NULL,
       instruction VARCHAR NOT NULL,
       status VARCHAR NOT NULL DEFAULT 'Status::Pending'
);
