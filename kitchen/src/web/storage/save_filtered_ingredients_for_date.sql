insert into filtered_ingredients(user_id, name, form, measure_type, plan_date)
    values (?, ?, ?, ?, ?) on conflict(user_id, name, form, measure_type, plan_date) DO NOTHING