-- Add up migration script here
create table recipes(user_id TEXT NOT NULL, recipe_id TEXT NOT NULL, recipe_text TEXT,
    constraint recipe_primary_key primary key (user_id, recipe_id));
create table categories(user_id TEXT NOT NULL, category_text TEXT,
    constraint category_primary_key primary key (user_id));