-- Add up migration script here
CREATE TABLE plans(id NUMBER, user_id TEXT, date TEXT);
CREATE table plan_recipes(plan_id NUMBER, recipe_id TEXT);