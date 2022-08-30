-- Add migration script here
CREATE TABLE sessions(id TEXT PRIMARY KEY, session_value BLOB NOT NULL);
CREATE TABLE users(id TEXT PRIMARY KEY, password_hashed TEXT NOT NULL);