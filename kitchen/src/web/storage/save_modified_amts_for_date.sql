insert into modified_amts(user_id, name, form, measure_type, amt, plan_date)
    values (?, ?, ?, ?, ?, ?) on conflict (user_id, name, form, measure_type, plan_date) do update set amt=excluded.amt