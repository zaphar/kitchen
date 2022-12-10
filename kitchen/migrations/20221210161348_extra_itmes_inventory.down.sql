-- Add down migration script here
drop table extra_items;

-- make a copy of of the filtered_ingredients table with only latest plan_date rows
create temp table TEMP_filtered_ingredients_copy as
    select
        user_id,
        name,
        max(plan_date) as plan_date,
        form,
        measure_type
    from filtered_ingredients
    group by user_id, name, form, measure_type;

-- Drop the filtered ingredients table and recreate without plan_date
drop table filtered_ingredients;
create table filtered_ingredients(
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    form TEXT NOT NULL,
    measure_type TEXT NOT NULL,
    primary key(user_id, name, form, measure_type)
);

-- Populate the new filtered ingredients table from the copied table
insert into filtered_ingredients
    select user_id, name, form, measure_type
    from TEMP_filtered_ingredients_copy;

-- make a copy of of the modified_amts table with only latest plan_date rows
create temp table TEMP_modified_amts_copy as
    select
        user_id,
        name,
        form,
        measure_type,
        max(plan_date) as plan_date,
        amt
    from modified_amts;

-- Drop modified_amts and recreate without plan_date.
drop table modified_amts;
create table modified_amts(
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    form TEXT NOT NULL,
    measure_type TEXT NOT NULL,
    amt TEXT NOT NULL, 
    primary key(user_id, name, form, measure_type)
);

-- Populate the new modified amts with rows from the copy.
insert into modified_amts
    select user_id, name, form, measure_type, amt
    from TEMP_modified_amts_copy;

drop table TEMP_modified_amts_copy;