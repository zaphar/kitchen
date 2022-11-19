insert into modified_amts(user_id, name, form, measure_type, amt)
    values (?, ?, ?, ?, ?) on conflict (user_id, name, form, measure_type) do update set amt=excluded.amt