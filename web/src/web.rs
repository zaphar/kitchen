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
use crate::{console_debug, console_error, console_log};
use reqwasm::http;
use sycamore::context::{use_context, ContextProvider, ContextProviderProps};
use sycamore::futures::spawn_local_in_scope;
use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router, RouterProps};

use recipes::{parse, Recipe};

#[derive(Clone)]
struct AppService {
    recipes: Signal<Vec<(usize, Recipe)>>,
}

impl AppService {
    fn new() -> Self {
        Self {
            recipes: Signal::new(Vec::new()),
        }
    }

    async fn fetch_recipes() -> Result<Vec<(usize, Recipe)>, String> {
        let resp = match http::Request::get("/api/v1/recipes").send().await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Error: {}", e)),
        };
        if resp.status() != 200 {
            return Err(format!("Status: {}", resp.status()));
        } else {
            console_debug!("We got a valid response back!");
            let recipe_list = match resp.json::<Vec<String>>().await {
                Ok(recipes) => recipes,
                Err(e) => return Err(format!("Eror getting recipe list as json {}", e)),
            };
            let mut parsed_list = Vec::new();
            for r in recipe_list {
                let recipe = match parse::as_recipe(&r) {
                    Ok(r) => r,
                    Err(e) => {
                        console_error!("Error parsing recipe {}", e);
                        break;
                    }
                };
                console_debug!("We parsed a recipe {}", recipe.title);
                parsed_list.push(recipe);
            }
            return Ok(parsed_list.drain(0..).enumerate().collect());
        }
    }

    fn get_recipes(&self) -> Signal<Vec<(usize, Recipe)>> {
        self.recipes.clone()
    }

    fn set_recipes(&mut self, recipes: Vec<(usize, Recipe)>) {
        self.recipes.set(recipes);
    }
}

/// Component to list available recipes.
#[component(RecipeList<G>)]
fn recipe_list() -> View<G> {
    let app_service = use_context::<AppService>();

    let titles = create_memo(cloned!(app_service => move || {
        app_service.get_recipes().get().iter().map(|(i, r)| (*i, r.title.clone())).collect::<Vec<(usize, String)>>()
    }));
    view! {
        ul {
            Keyed(KeyedProps{
                iterable: titles,
                template: |(i, title)| {
                    view! { li(on:click=move |_| {
                        console_log!("clicked item with index: {}", i)
                    }) { (title) } }
                },
                key: |(i, title)| (*i, title.clone()),
            })
        }
    }
}

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
        Router(RouterProps::new(HistoryIntegration::new(), move |routes: ReadSignal<AppRoutes>| {
            let t = create_memo(cloned!((app_service) => move || {
                console_debug!("Determining route.");
                let route = routes.get();
                console_debug!("Route {:?}", route);
                match route.as_ref() {
                    AppRoutes::Root => view! {
                        div { "hello chefs!" }
                        ContextProvider(ContextProviderProps {
                                value: app_service.clone(),
                                children: || view! { RecipeList() }
                        })
                    },
                    AppRoutes::Recipe{index:_idx} => view! {
                        "TODO!!"
                    },
                    AppRoutes::Menu => view! {
                        "TODO!!"
                    },
                    AppRoutes::NotFound => view! {
                        "NotFound"
                    }
                }
            }));
            console_debug!("Created our route view memo.");
            view! {
                div(class="app") {
                    (t.get().as_ref().clone())
                }
            }
        }))
    }
}
