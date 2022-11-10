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
use sycamore::prelude::*;

pub mod cook;
pub mod inventory;
pub mod plan;

pub use cook::*;
pub use inventory::*;
pub use plan::*;

#[derive(Props)]
pub struct PageState<'a, G: Html> {
    pub children: Children<'a, G>,
    pub selected: Option<String>,
}

#[component]
pub fn PlanningPage<'a, G: Html>(cx: Scope<'a>, state: PageState<'a, G>) -> View<G> {
    let PageState { children, selected } = state;
    let children = children.call(cx);
    let planning_tabs: Vec<(String, &'static str)> = vec![
        ("/ui/planning/plan".to_owned(), "Plan"),
        ("/ui/planning/inventory".to_owned(), "Inventory"),
        ("/ui/planning/cook".to_owned(), "Cook"),
    ];

    view! {cx,
        TabbedView(
            selected=selected,
            tablist=planning_tabs,
        ) { (children) }
    }
}
