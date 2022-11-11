-- Add up migration script here
create table filtered_ingredients(
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    form TEXT NOT NULL,
    measure_type TEXT NOT NULL,
    primary key(user_id, name, form, measure_type)
);

create table modified_amts(
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    form TEXT NOT NULL,
    measure_type TEXT NOT NULL,
    amt TEXT NOT NULL, 
    primary key(user_id, name, form, measure_type)
);