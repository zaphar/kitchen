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

#[derive(Props, PartialEq)]
struct RecipeListProps {
    recipe_list: Vec<Recipe>,
}

/// Component to list available recipes.
fn RecipeList(cx: Scope) -> Element {
    let props = use_state(&cx, || RecipeListProps {
        recipe_list: vec![],
    });

    use_future(&cx, || {
        let props = props.for_async();
        async move {
            let req = http::Request::get("/api/v1/recipes").send().await;
            match req {
                Ok(resp) => {
                    if resp.status() != 200 {
                        console_log!("Status: {}", resp.status());
                    } else {
                        console_log!("We got a valid response back!");
                        let recipe_list = match resp.json::<Vec<String>>().await {
                            Ok(recipes) => recipes,
                            Err(e) => {
                                console_log!("Eror getting recipe list as json {}", e);
                                Vec::new()
                            }
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
                        props.set(RecipeListProps {
                            recipe_list: parsed_list,
                        });
                    }
                }
                Err(e) => {
                    console_log!("Error: {}", e);
                }
            }
        }
    });
    cx.render(rsx! {
        ul {
            (&props.recipe_list).into_iter().map(|i| {
                let title = &i.title;
                rsx!(li { "{title}" })
             })
        }
    })
}

pub fn ui(cx: Scope) -> Element {
    cx.render(rsx! {
        div { "hello chefs!" }
        RecipeList { }
    })
}
