// Copyright 2022 Jeremy Wall (Jeremy@marzhilsltudios.com)
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
use crate::components::tabs::*;
use chrono::NaiveDate;
use sycamore::prelude::*;

pub mod cook;
pub mod inventory;
pub mod plan;
pub mod select;

pub use cook::*;
pub use inventory::*;
pub use plan::*;
pub use select::*;

#[derive(Props)]
pub struct PageState<'ctx, G: Html> {
    pub children: Children<'ctx, G>,
    pub selected: Option<String>,
    pub plan_date: &'ctx ReadSignal<Option<NaiveDate>>,
}

#[component]
pub fn PlanningPage<'ctx, G: Html>(cx: Scope<'ctx>, state: PageState<'ctx, G>) -> View<G> {
    let PageState {
        children,
        selected,
        plan_date,
    } = state;
    let children = children.call(cx);
    let planning_tabs: Vec<(String, &'static str)> = vec![
        ("/ui/planning/select".to_owned(), "Select"),
        ("/ui/planning/plan".to_owned(), "Plan"),
        ("/ui/planning/inventory".to_owned(), "Inventory"),
        ("/ui/planning/cook".to_owned(), "Cook"),
    ];

    view! {cx,
        TabbedView(
            selected=selected,
            tablist=planning_tabs,
        ) { div {
                "Plan Date: " (plan_date.get().map_or(String::from("Unknown"), |d| format!("{}", d)))
            }
            (children)
        }
    }
}
