with latest_dates as (
    select user_id, max(date(plan_date)) as plan_date from plan_recipes
    where user_id = ?
    group by user_id
)

select
    extra_items.name,
    extra_items.amt
from latest_dates
inner join extra_items on
    latest_dates.user_id = extra_items.user_id
    and latest_dates.plan_date = extra_items.plan_date