-- Add up migration script here
-- First we collect a safe set of the data deduped for the unique index to handle using max to select a winning count.
create temp table TEMP_plan_recipes_deduped as
    select user_id, plan_date, recipe_id, max(count) as count
    from plan_recipes
    group by user_id, plan_date, recipe_id;

-- Then we drop the plan_recipes from the table
delete from plan_recipes;

-- Create the unique index
create unique index mealplan_lookup_index
    on plan_recipes (user_id, plan_date, recipe_id);

-- And finally insert the dedeuped records back into the table before dropping the temp table.
insert into plan_recipes
    select user_id, plan_date, recipe_id, count
    from TEMP_plan_recipes_deduped;
drop table TEMP_plan_recipes_deduped;