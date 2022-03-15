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
use crate::app_state::AppRoutes;
use crate::components::{shopping_list::*, tabs::*};
use crate::pages::PageState;

use sycamore::prelude::*;

#[derive(Clone)]
pub struct InventoryPageProps {
    pub page_state: PageState,
}

#[component(InventoryPage<G>)]
pub fn inventory_page(props: InventoryPageProps) -> View<G> {
    let route_signal = props.page_state.route.clone();
    view! {
        TabbedView(TabState {
            route: props.page_state.route.clone(),
            inner: view! {
                ShoppingList()
                div {
                    a(href="#", on:click=move |_| {
                     route_signal.set(AppRoutes::Cook);
                    }) { "Next" }
                }
            },
        })
    }
}
