-- Add up migration script here
create table staples (
    user_id TEXT NOT NULL,
    content TEXT NOT NULL,
    primary key(user_id)
);