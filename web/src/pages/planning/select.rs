// Copyright 2023 Jeremy Wall (Jeremy@marzhilsltudios.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use super::PlanningPage;
use crate::{
    app_state::{Message, StateHandler},
    components::PlanList,
};

use chrono::NaiveDate;
use sycamore::prelude::*;

#[component]
pub fn SelectPage<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let plan_dates = sh.get_selector(cx, |state| {
        let mut plans = state
            .get()
            .plan_dates
            .iter()
            .cloned()
            .collect::<Vec<NaiveDate>>();
        plans.sort_unstable_by(|d1, d2| d2.cmp(d1));
        plans
    });
    view! {cx,
        PlanningPage(
            selected=Some("Select".to_owned()),
        ) {
            PlanList(sh=sh, list=plan_dates)
            button(on:click=move |_| {
                sh.dispatch(cx, Message::SelectPlanDate(chrono::offset::Local::now().naive_local().date(), Some(Box::new(|| {
                    sycamore_router::navigate("/ui/planning/plan");
                }))))
            }) {
                "Start Plan for Today"
            }
        }
    }
}
