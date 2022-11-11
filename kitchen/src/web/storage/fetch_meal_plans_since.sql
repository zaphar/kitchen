select plan_date as "plan_date: NaiveDate", recipe_id, count
from plan_recipes
where
    user_id = ?
    and date(plan_date) > ?
order by user_id, plan_date