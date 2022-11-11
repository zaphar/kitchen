with max_date as (
    select user_id, max(date(plan_date)) as plan_date from plan_recipes group by user_id
)

select plan_recipes.plan_date as "plan_date: NaiveDate", plan_recipes.recipe_id, plan_recipes.count
    from plan_recipes
    inner join max_date on plan_recipes.user_id = max_date.user_id
where
    plan_recipes.user_id = ?
    and plan_recipes.plan_date = max_date.plan_date