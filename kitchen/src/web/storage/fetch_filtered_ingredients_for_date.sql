select
    filtered_ingredients.name,
    filtered_ingredients.form,
    filtered_ingredients.measure_type
from filtered_ingredients
where
     user_id = ?
     and plan_date = ?