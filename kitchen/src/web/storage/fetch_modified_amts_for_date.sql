select
    modified_amts.name,
    modified_amts.form,
    modified_amts.measure_type,
    modified_amts.amt
from modified_amts
where
    user_id = ?
    and plan_date = ?