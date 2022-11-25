-- Add up migration script here
create unique index mealplan_lookup_index on plan_recipes (user_id, plan_date, recipe_id);