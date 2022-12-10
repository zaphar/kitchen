insert into extra_items (user_id, name, plan_date, amt)
values (?, ?, date(), ?)
on conflict (user_id, name, plan_date) do update set amt=excluded.amt