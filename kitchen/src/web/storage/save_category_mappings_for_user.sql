insert into category_mappings
    (user_id, ingredient_name, category_name)
    values (?, ?, ?)
    on conflict (user_id, ingredient_name)
        do update set category_name=excluded.category_name
