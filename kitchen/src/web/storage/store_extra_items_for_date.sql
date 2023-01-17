insert into extra_items (user_id, name, amt, plan_date)
values (?, ?, ?, ?)
on conflict (user_id, name, plan_date) do update set amt=excluded.amt