-- Add up migration script here

-- Create our extra items table
create table extra_items(
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    plan_date DATE NOT NULL,
    amt TEXT NOT NULL,
    primary key(user_id, name, plan_date)
);

-- Store a copy of filtered ingredients with current date as plan_date
create temp table TEMP_filtered_ingredients_copy as 
    select
        user_id,
        name,
        date() as plan_date,
        form,
        measure_type
    from filtered_ingredients;

-- Drop the filtered ingredients table and recreate with plan_date in the primary key
drop table filtered_ingredients;
create table filtered_ingredients(
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    form TEXT NOT NULL,
    measure_type TEXT NOT NULL,
    plan_date DATE NOT NULL,
    primary key(user_id, name, form, measure_type, plan_date)
);

-- Populate the new filtered ingredients table from the copied table
insert into filtered_ingredients
    select user_id, name, form, measure_type, plan_date
    from TEMP_filtered_ingredients_copy;

drop table TEMP_filtered_ingredients_copy;

-- make a copy of of the modified_amts table with current date as plan_date
create temp table TEMP_modified_amts_copy as
    select
        user_id,
        name,
        form,
        measure_type,
        date() as plan_date,
        amt
    from modified_amts;

-- Drop modified_amts and recreate with plan_date as part of primary key.
drop table modified_amts;
create table modified_amts(
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    form TEXT NOT NULL,
    measure_type TEXT NOT NULL,
    plan_date DATE NOT NULL,
    amt TEXT NOT NULL, 
    primary key(user_id, name, form, measure_type, plan_date)
);

-- Populate the new modified amts with rows from the copy.
insert into modified_amts
    select user_id, name, form, measure_type, plan_date, amt
    from TEMP_modified_amts_copy;

drop table TEMP_modified_amts_copy;
