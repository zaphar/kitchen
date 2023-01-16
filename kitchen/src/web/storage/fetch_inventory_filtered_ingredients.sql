with latest_dates as (
    select user_id, max(date(plan_date)) as plan_date from plan_recipes
    where user_id = ?
    group by user_id
)

select
    filtered_ingredients.name,
    filtered_ingredients.form,
    filtered_ingredients.measure_type
from latest_dates
inner join filtered_ingredients on
     latest_dates.user_id = filtered_ingredients.user_id
     and latest_dates.plan_date = filtered_ingredients.plan_date