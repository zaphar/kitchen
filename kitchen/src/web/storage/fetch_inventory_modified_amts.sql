with latest_dates as (
    select user_id, max(date(plan_date)) as plan_date from plan_recipes
    where user_id = ?
    group by user_id
)

select
    modified_amts.name,
    modified_amts.form,
    modified_amts.measure_type,
    modified_amts.amt
from latest_dates
inner join modified_amts on
    latest_dates.user_id = modified_amts.user_id
    and latest_dates.plan_date = modified_amts.plan_date