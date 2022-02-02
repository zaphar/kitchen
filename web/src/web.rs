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
use crate::{components::*, service::AppService};
use crate::{console_debug, console_error, console_log};

use sycamore::{
    context::{ContextProvider, ContextProviderProps},
    futures::spawn_local_in_scope,
    prelude::*,
};
use sycamore_router::{HistoryIntegration, Route, Router, RouterProps};

#[derive(Route, Debug)]
enum AppRoutes {
    #[to("/ui")]
    Root,
    #[to("/ui/recipe/<index>")]
    Recipe { index: usize },
    #[to("/ui/menu")]
    Menu,
    #[not_found]
    NotFound,
}

#[component(UI<G>)]
pub fn ui() -> View<G> {
    let app_service = AppService::new();
    console_log!("Starting UI");
    create_effect(cloned!((app_service) => move || {
        spawn_local_in_scope({
            let mut app_service = app_service.clone();
            async move {
                match AppService::fetch_recipes().await {
                    Ok(recipes) => {
                        app_service.set_recipes(recipes);
                    }
                    Err(msg) => console_error!("Failed to get recipes {}", msg),
                }
            }
        });
    }));
    view! {
        // NOTE(jwall): Set the app_service in our toplevel scope. Children will be able
        // to find the service as long as they are a child of this scope.
        ContextProvider(ContextProviderProps {
                value: app_service.clone(),
                children: || view! {
                    Router(RouterProps::new(HistoryIntegration::new(), move |routes: ReadSignal<AppRoutes>| {
                        let t = create_memo(move || {
                            console_debug!("Determining route.");
                            let route = routes.get();
                            console_debug!("Route {:?}", route);
                            match route.as_ref() {
                                AppRoutes::Root => view! {
                                    Start()
                                },
                                AppRoutes::Recipe{index:idx} => view! {
                                        RecipeView(*idx)
                                },
                                AppRoutes::Menu => view! {
                                    "TODO!!"
                                },
                                AppRoutes::NotFound => view! {
                                    "NotFound"
                                }
                            }
                        });
                        console_debug!("Created our route view memo.");
                        view! {
                            // NOTE(jwall): The Router component *requires* there to be exactly one node as the root of this view.
                            // No fragments or missing nodes allowed or it will panic at runtime.
                            div(class="app") {
                                (t.get().as_ref().clone())
                            }
                        }
                    }))
                }
        })
    }
}
