CREATE TABLE IF NOT EXISTS Users (
       user_id BIGSERIAL PRIMARY KEY,
       user_name VARCHAR NOT NULL UNIQUE,
       password_hash VARCHAR NOT NULL,
       robot_serial_number VARCHAR NOT NULL UNIQUE REFERENCES Robot(robot_serial_number)
);
