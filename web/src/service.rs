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
use crate::{console_debug, console_error};

use reqwasm::http;
use sycamore::prelude::*;

use recipes::{parse, Recipe};

#[derive(Clone)]
pub struct AppService {
    // TODO(jwall): Should each Recipe also be a Signal?
    recipes: Signal<Vec<(usize, Recipe)>>,
}

impl AppService {
    pub fn new() -> Self {
        Self {
            recipes: Signal::new(Vec::new()),
        }
    }

    pub async fn fetch_recipes() -> Result<Vec<(usize, Recipe)>, String> {
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

    pub fn get_recipes(&self) -> Signal<Vec<(usize, Recipe)>> {
        self.recipes.clone()
    }

    pub fn set_recipes(&mut self, recipes: Vec<(usize, Recipe)>) {
        self.recipes.set(recipes);
    }
}
