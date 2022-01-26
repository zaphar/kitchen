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
#![allow(non_snake_case)]

use crate::console_log;
use reqwasm::http;
use sycamore::context::{use_context, ContextProvider, ContextProviderProps};
use sycamore::futures::spawn_local_in_scope;
use sycamore::prelude::*;

use recipes::{parse, Recipe};

#[derive(Clone)]
struct AppService {
    recipes: Signal<Vec<Recipe>>,
}

impl AppService {
    fn new() -> Self {
        Self {
            recipes: Signal::new(Vec::new()),
        }
    }

    async fn fetch_recipes() -> Result<Vec<Recipe>, String> {
        let resp = match http::Request::get("/api/v1/recipes").send().await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Error: {}", e)),
        };
        if resp.status() != 200 {
            return Err(format!("Status: {}", resp.status()));
        } else {
            console_log!("We got a valid response back!");
            let recipe_list = match resp.json::<Vec<String>>().await {
                Ok(recipes) => recipes,
                Err(e) => return Err(format!("Eror getting recipe list as json {}", e)),
            };
            let mut parsed_list = Vec::new();
            for r in recipe_list {
                let recipe = match parse::as_recipe(&r) {
                    Ok(r) => r,
                    Err(e) => {
                        console_log!("Error parsing recipe {}", e);
                        break;
                    }
                };
                console_log!("We parsed a recipe {}", recipe.title);
                parsed_list.push(recipe);
            }
            // TODO(jwall): It would appear that their API doesn't support this
            // model for async operations.
            //self.recipes = parsed_list;
            return Ok(parsed_list);
        }
    }

    fn get_recipes(&self) -> Signal<Vec<Recipe>> {
        self.recipes.clone()
    }

    fn set_recipes(&mut self, recipes: Vec<Recipe>) {
        self.recipes.set(recipes);
    }
}

/// Component to list available recipes.
#[component(RecipeList<G>)]
fn recipe_list() -> View<G> {
    let props = use_context::<AppService>();

    view! {
        ul {
            Indexed(IndexedProps{
                iterable: props.get_recipes().handle(),
                template: |recipe| {
                    view! { li { (recipe.title) } }
                }
            })
        }
    }
}

#[component(UI<G>)]
pub fn ui() -> View<G> {
    let app_state = AppService::new();

    spawn_local_in_scope({
        let mut app_state = app_state.clone();
        async move {
            match AppService::fetch_recipes().await {
                Ok(recipes) => {
                    app_state.set_recipes(recipes);
                }
                Err(msg) => console_log!("Failed to get recipes {}", msg),
            }
        }
    });
    view! {
        div { "hello chefs!" }
        ContextProvider(ContextProviderProps {
                value: app_state,
                children: || view! { RecipeList() }
        })
    }
}
