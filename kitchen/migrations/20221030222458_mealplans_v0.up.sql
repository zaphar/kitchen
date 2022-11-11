-- Add up migration script here
CREATE table plan_recipes(user_id TEXT NOT NULL, plan_date DATE NOT NULL, recipe_id TEXT NOT NULL, count integer NOT NULL);