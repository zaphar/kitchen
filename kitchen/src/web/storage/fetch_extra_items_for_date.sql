select
    name,
    amt
from extra_items
where
    user_id = ?
    and plan_date = ?