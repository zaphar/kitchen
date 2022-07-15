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
use crate::pages::*;
use crate::{app_state::*, components::*, router_integration::*, service::AppService};
use crate::{console_error, console_log};

use sycamore::{
    context::{ContextProvider, ContextProviderProps},
    futures::spawn_local_in_scope,
    prelude::*,
};

fn route_switch<G: Html>(route: ReadSignal<AppRoutes>) -> View<G> {
    // NOTE(jwall): This needs to not be a dynamic node. The rules around
    // this are somewhat unclear and underdocumented for Sycamore. But basically
    // avoid conditionals in the `view!` macro calls here.
    cloned!((route) => match route.get().as_ref() {
        AppRoutes::Plan => view! {
            PlanPage()
        },
        AppRoutes::Inventory => view! {
            InventoryPage()
        },
        AppRoutes::Cook => view! {
            CookPage()
        },
        AppRoutes::Recipe(idx) => view! {
            RecipePage(RecipePageProps { recipe: Signal::new(*idx) })
        },
        AppRoutes::NotFound => view! {
            // TODO(Create a real one)
            PlanPage()
        },
        AppRoutes::Error(ref e) => {
            let e = e.clone();
            view! {
                "Error: " (e)
            }
        }
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
                create_effect(move || {
                    spawn_local_in_scope({
                        let mut app_service = app_service.clone();
                        async move {
                            match AppService::fetch_recipes_from_storage() {
                                Ok((_, Some(recipes))) => {
                                    app_service.set_recipes(recipes);
                                }
                                Ok((_, None)) => {
                                    console_error!("No recipes to find");
                                }
                                Err(msg) => console_error!("Failed to get recipes {}", msg),
                            }
                        }
                    });
                });

                view! {
                    div(class="app") {
                        Header()
                        Router(RouterProps {
                            route: AppRoutes::Plan,
                            route_select: route_switch,
                            browser_integration: BrowserIntegration::new(),
                        })
                    }
                }
            }
        })
    }
}
