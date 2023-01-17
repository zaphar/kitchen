select plan_date as "plan_date: NaiveDate", recipe_id, count
    from plan_recipes
where
    user_id = ?
    and plan_date = ?