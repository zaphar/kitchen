insert into staples (user_id, content) values (?, ?)
    on conflict(user_id) do update set content = excluded.content