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
use crate::{app_state::*, components::*, service::AppService};
use crate::{console_debug, console_error, console_log};

use sycamore::{
    context::{ContextProvider, ContextProviderProps},
    futures::spawn_local_in_scope,
    prelude::*,
};

use crate::pages::*;

fn route_switch<G: Html>(page_state: PageState) -> View<G> {
    let route = page_state.route.clone();
    cloned!((page_state, route) => view! {
        (match route.get().as_ref() {
            AppRoutes::Plan => view! {
                PlanPage(PlanPageProps { page_state: page_state.clone() })
            },
            AppRoutes::Inventory => view! {
                InventoryPage(InventoryPageProps { page_state: page_state.clone() })
            },
            AppRoutes::Cook => view! {
                CookPage(CookPageProps { page_state: page_state.clone() })
            },
        })
    })
}

#[component(UI<G>)]
pub fn ui() -> View<G> {
    let app_service = AppService::new();
    console_log!("Starting UI");
    view! {
        // NOTE(jwall): Set the app_service in our toplevel scope. Children will be able
        // to find the service as long as they are a child of this scope.
        ContextProvider(ContextProviderProps {
            value: app_service.clone(),
            children: || {
                let view = Signal::new(View::empty());
                let route = Signal::new(AppRoutes::Plan);
                let page_state = PageState { route: route.clone() };
                create_effect(cloned!((page_state, view) => move || {
                    spawn_local_in_scope(cloned!((page_state, view) => {
                        let mut app_service = app_service.clone();
                        async move {
                            match AppService::fetch_recipes().await {
                                Ok((_, Some(recipes))) => {
                                    app_service.set_recipes(recipes);
                                }
                                Ok((_, None)) => {
                                    console_error!("No recipes to find");
                                }
                                Err(msg) => console_error!("Failed to get recipes {}", msg),
                            }
                            console_debug!("Determining route.");
                            view.set(route_switch(page_state.clone()));
                            console_debug!("Created our route view effect.");
                        }
                    }));
                }));
                view! {
                    // NOTE(jwall): The Router component *requires* there to be exactly one node as the root of this view.
                    // No fragments or missing nodes allowed or it will panic at runtime.
                    div(class="app") {
                        Header()
                        (view.get().as_ref().clone())
                    }
                }
            }
        })
    }
}
