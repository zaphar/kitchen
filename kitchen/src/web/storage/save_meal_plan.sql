insert into plan_recipes (user_id, plan_date, recipe_id, count) values (?, ?, ?, ?)
    on conflict (user_id, plan_date, recipe_id) do update set count=excluded.count;