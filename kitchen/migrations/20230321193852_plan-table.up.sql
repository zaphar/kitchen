-- Add up migration script here
create temp table TEMP_plan_dates_deduped AS
    select distinct user_id, plan_date from plan_recipes;

create table plan_table (user_id TEXT NOT NULL, plan_date TEXT NOT NULL, primary key (user_id, plan_date) );

insert into plan_table
    select user_id, plan_date from TEMP_plan_dates_deduped;

drop table TEMP_plan_dates_deduped;