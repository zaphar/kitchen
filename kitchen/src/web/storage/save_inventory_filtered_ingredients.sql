insert into filtered_ingredients(user_id, name, form, measure_type, plan_date)
    values (?, ?, ?, ?, date()) on conflict(user_id, name, form, measure_type, plan_date) DO NOTHING