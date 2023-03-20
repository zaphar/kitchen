insert into plan_table (user_id, plan_date) values (?, ?)
    on conflict (user_id, plan_date) do nothing;