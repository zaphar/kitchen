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
use tracing::{debug, error, info, instrument};

use recipe_store::{self, *};
use sycamore::{
    context::{ContextProvider, ContextProviderProps},
    futures::spawn_local_in_scope,
    prelude::*,
};

#[instrument]
fn route_switch<G: Html>(route: ReadSignal<AppRoutes>) -> View<G> {
    // NOTE(jwall): This needs to not be a dynamic node. The rules around
    // this are somewhat unclear and underdocumented for Sycamore. But basically
    // avoid conditionals in the `view!` macro calls here.
    let node = cloned!((route) => match route.get().as_ref() {
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
            RecipePage(RecipePageProps { recipe: Signal::new(idx.clone()) })
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
    });
    debug!(node=?node, "routing to new page");
    node
}

#[cfg(not(target_arch = "wasm32"))]
fn get_appservice() -> AppService<AsyncFileStore> {
    AppService::new(recipe_store::AsyncFileStore::new("/".to_owned()))
}
#[cfg(target_arch = "wasm32")]
fn get_appservice() -> AppService<HttpStore> {
    AppService::new(recipe_store::HttpStore::new("/api/v1".to_owned()))
}

#[instrument]
#[component(RouterComponent<G>)]
fn router_component(route: Option<AppRoutes>) -> View<G> {
    debug!("Setting up router component");
    match route {
        Some(route) => {
            view! {
                StaticRouter(StaticRouterProps{route: route, route_select: route_switch})
            }
        }
        None => {
            view! {
                Router(RouterProps{
                    route: AppRoutes::default(),
                    browser_integration: BrowserIntegration::new(),
                    route_select: route_switch,
                })
            }
        }
    }
}

#[instrument]
#[component(UI<G>)]
pub fn ui(route: Option<AppRoutes>) -> View<G> {
    let app_service = get_appservice();
    info!("Starting UI");
    view! {
        // NOTE(jwall): Set the app_service in our toplevel scope. Children will be able
        // to find the service as long as they are a child of this scope.
        ContextProvider(ContextProviderProps {
            value: app_service.clone(),
            children: || {
                let view = Signal::new(View::empty());
                create_effect(cloned!((route, view, app_service) => move || {
                    spawn_local_in_scope({
                        cloned!((route, view, app_service) => async move {
                            debug!("fetching recipes");
                            if let Err(msg) =  app_service.init(false).await {
                                error!("Failed to get recipes {}", msg);
                            }
                            view.set(view!{
                                RouterComponent(route)
                            });
                        })
                    })
                }));

                view! {
                    div(class="app") {
                        Header()
                        (view.get().as_ref().clone())
                    }
                }
            }
        })
    }
}
