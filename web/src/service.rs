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
use std::collections::BTreeMap;

use crate::{console_debug, console_error, console_log};

use reqwasm::http::{self};
use sycamore::prelude::*;
use web_sys::{window, Storage};

use recipes::{parse, Ingredient, IngredientAccumulator, IngredientKey, Recipe};

#[derive(Clone)]
pub struct AppService {
    recipes: Signal<Vec<(usize, Signal<Recipe>)>>,
    menu_list: Signal<BTreeMap<usize, usize>>,
}

impl AppService {
    pub fn new() -> Self {
        Self {
            recipes: Signal::new(Vec::new()),
            menu_list: Signal::new(BTreeMap::new()),
        }
    }

    fn get_storage() -> Result<Option<Storage>, String> {
        window()
            .unwrap()
            .local_storage()
            .map_err(|e| format!("{:?}", e))
    }

    async fn fetch_recipes_http() -> Result<String, String> {
        let resp = match http::Request::get("/api/v1/recipes").send().await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Error: {}", e)),
        };
        if resp.status() != 200 {
            return Err(format!("Status: {}", resp.status()));
        } else {
            console_debug!("We got a valid response back!");
            return Ok(resp.text().await.map_err(|e| format!("{}", e))?);
        }
    }

    pub async fn synchronize_recipes() -> Result<(), String> {
        console_log!("Synchronizing Recipes");
        let storage = Self::get_storage()?.unwrap();
        let recipes = Self::fetch_recipes_http().await?;
        storage
            .set_item("recipes", &recipes)
            .map_err(|e| format!("{:?}", e))?;
        Ok(())
    }

    pub fn fetch_recipes_from_storage() -> Result<Option<Vec<(usize, Recipe)>>, String> {
        let storage = Self::get_storage()?.unwrap();
        match storage
            .get_item("recipes")
            .map_err(|e| format!("{:?}", e))?
        {
            Some(s) => {
                let parsed =
                    serde_json::from_str::<Vec<String>>(&s).map_err(|e| format!("{}", e))?;
                let mut parsed_list = Vec::new();
                for r in parsed {
                    let recipe = match parse::as_recipe(&r) {
                        Ok(r) => r,
                        Err(e) => {
                            console_error!("Error parsing recipe {}", e);
                            continue;
                        }
                    };
                    console_debug!("We parsed a recipe {}", recipe.title);
                    parsed_list.push(recipe);
                }
                Ok(Some(parsed_list.drain(0..).enumerate().collect()))
            }
            None => Ok(None),
        }
    }

    pub async fn fetch_recipes() -> Result<Option<Vec<(usize, Recipe)>>, String> {
        if let Some(recipes) = Self::fetch_recipes_from_storage()? {
            return Ok(Some(recipes));
        } else {
            console_debug!("No recipes in cache synchronizing from api");
            // Try to synchronize first
            Self::synchronize_recipes().await?;
            Ok(Self::fetch_recipes_from_storage()?)
        }
    }

    pub async fn refresh_recipes(&mut self) -> Result<(), String> {
        Self::synchronize_recipes().await?;
        if let Some(r) = Self::fetch_recipes().await? {
            self.set_recipes(r);
        }
        Ok(())
    }

    pub fn get_recipe_by_index(&self, idx: usize) -> Option<Signal<Recipe>> {
        self.recipes.get().get(idx).map(|(_, r)| r.clone())
    }

    pub fn get_shopping_list(&self) -> BTreeMap<IngredientKey, Ingredient> {
        let mut acc = IngredientAccumulator::new();
        let recipe_counts = self.menu_list.get();
        for (idx, count) in recipe_counts.iter() {
            for _ in 0..*count {
                acc.accumulate_from(self.get_recipe_by_index(*idx).unwrap().get().as_ref());
            }
        }
        acc.ingredients()
    }

    pub fn set_recipe_count_by_index(&mut self, i: usize, count: usize) {
        let mut v = (*self.menu_list.get()).clone();
        v.insert(i, count);
        self.menu_list.set(v);
    }

    pub fn get_recipe_count_by_index(&self, i: usize) -> usize {
        self.menu_list.get().get(&i).map(|i| *i).unwrap_or_default()
    }

    pub fn get_recipes(&self) -> Signal<Vec<(usize, Signal<Recipe>)>> {
        self.recipes.clone()
    }

    pub fn get_menu_list(&self) -> Vec<(usize, usize)> {
        self.menu_list
            .get()
            .iter()
            // We exclude recipes in the menu_list with count 0
            .filter(|&(_, count)| *count != 0)
            .map(|(idx, count)| (*idx, *count))
            .collect()
    }

    pub fn set_recipes(&mut self, mut recipes: Vec<(usize, Recipe)>) {
        self.recipes.set(
            recipes
                .drain(0..)
                .map(|(i, r)| (i, Signal::new(r)))
                .collect(),
        );
    }
}
