-- Add up migration script here

create table category_mappings(
    user_id TEXT NOT NULL,
    ingredient_name TEXT NOT NULL,
    category_name TEXT NOT NULL DEFAULT "Misc",
    primary key(user_id, ingredient_name)
);

create index user_category_lookup on category_mappings (user_id, category_name);