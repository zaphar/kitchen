select distinct plan_date as "plan_date: NaiveDate" from plan_recipes
where user_id = ?