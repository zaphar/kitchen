// Copyright 2022 Jeremy Wall
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

use sycamore::prelude::*;

use crate::app_state::StateHandler;

#[component]
pub fn Header<'ctx, G: Html>(cx: Scope<'ctx>, h: StateHandler<'ctx>) -> View<G> {
    let login = h.get_selector(cx, |sig| match &sig.get().auth {
        Some(id) => id.user_id.clone(),
        None => "Login".to_owned(),
    });
    view! {cx,
        nav(class="no-print row-flex align-center header-bg heavy-bottom-border") {
            h1(class="title") { "Kitchen" }
            ul(class="row-flex align-center") {
                li { a(href="/ui/planning/select") { "MealPlan" } }
                li { a(href="/ui/manage/ingredients") { "Manage" } }
                li { a(href="/ui/login") { (login.get()) } }
            }
        }
    }
}
