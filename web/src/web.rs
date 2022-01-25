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
use std::iter::Iterator;

use crate::console_log;
use dioxus::prelude::*;
use reqwasm::http;

use recipes::{parse, Recipe};

#[derive(Props, PartialEq, Clone)]
struct AppService {
    recipes: Vec<Recipe>,
}

impl AppService {
    fn new() -> Self {
        Self {
            recipes: Vec::new(),
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

    fn get_recipes(&self) -> &Vec<Recipe> {
        &self.recipes
    }

    fn set_recipes(&mut self, recipes: Vec<Recipe>) {
        self.recipes = recipes;
    }
}

#[derive(Props)]
struct RecipeListProps<'a> {
    app_service: UseState<'a, AppService>,
}

/// Component to list available recipes.
fn recipe_list<'a>(cx: Scope<'a, RecipeListProps<'a>>) -> Element {
    let props = cx.props.app_service;

    cx.render(rsx! {
        ul {
            props.get_recipes().into_iter().map(|i| {
                let title = &i.title;
                rsx!(li { "{title}" })
             })
        }
    })
}

pub fn ui(cx: Scope) -> Element {
    let app_state = use_state(&cx, AppService::new);

    let fut = use_future(&cx, || async move { AppService::fetch_recipes().await });
    cx.render(rsx! {
        div { "hello chefs!" }
        {match fut.value() {
            Some(Ok(recipes)) => {
                app_state.modify().set_recipes(recipes.clone());
                rsx!{ recipe_list(app_service: app_state) }
            }
            Some(Err(e)) => {
                console_log!("{}", e);
                rsx!{ div { class: "error", "{e}" } }
            }
            None => {
                //panic!("We seem to have failed to execute our future.")
                rsx!{ div { "Loading recipe list..." }}
            }
        }}
    })
}
