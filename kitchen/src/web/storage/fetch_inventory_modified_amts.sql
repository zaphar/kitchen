with latest_dates as (
    select
        user_id,
        name,
        form,
        measure_type,
        amt,
        max(plan_date) as plan_date
    from modified_amts
    where user_id = ?
)

select
    modified_amts.name,
    modified_amts.form,
    modified_amts.measure_type,
    modified_amts.amt
from latest_dates
inner join modified_amts on
    latest_dates.user_id = modified_amts.user_id
    and latest_dates.name = modified_amts.name
    and latest_dates.form = modified_amts.form
    and latest_dates.amt = modified_amts.amt
    and latest_dates.plan_date = modified_amts.plan_date