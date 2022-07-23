-- Your SQL goes here

create table recipes (
    id text primary key, -- intended to be a UUID
    recipe_text text -- Recipe content
);

create table categories (
    id text primary key, -- intended to be a UUIDgg
    categories_text text -- category content
);