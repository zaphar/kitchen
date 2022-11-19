insert into filtered_ingredients(user_id, name, form, measure_type)
    values (?, ?, ?, ?) on conflict(user_id, name, form, measure_type) DO NOTHING