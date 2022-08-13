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
use std::collections::{BTreeMap, BTreeSet};

#[cfg(target_arch = "wasm32")]
use reqwasm::http;
use serde_json::{from_str, to_string};
use sycamore::{context::use_context, prelude::*};
use tracing::{debug, error, info, instrument, warn};
use web_sys::{window, Storage};

use recipe_store::{AsyncFileStore, RecipeStore};
use recipes::{parse, Ingredient, IngredientAccumulator, Recipe};

#[cfg(not(target_arch = "wasm32"))]
pub fn get_appservice_from_context() -> AppService<AsyncFileStore> {
    use_context::<AppService<AsyncFileStore>>()
}
#[cfg(target_arch = "wasm32")]
pub fn get_appservice_from_context() -> AppService<AsyncFileStore> {
    use_context::<AppService<HttpStore>>()
}

#[derive(Clone, Debug)]
pub struct AppService<S>
where
    S: RecipeStore,
{
    recipes: Signal<Vec<(usize, Signal<Recipe>)>>,
    staples: Signal<Option<Recipe>>,
    category_map: Signal<BTreeMap<String, String>>,
    menu_list: Signal<BTreeMap<usize, usize>>,
    store: S,
}

impl<S> AppService<S>
where
    S: RecipeStore,
{
    pub fn new(store: S) -> Self {
        Self {
            recipes: Signal::new(Vec::new()),
            staples: Signal::new(None),
            category_map: Signal::new(BTreeMap::new()),
            menu_list: Signal::new(BTreeMap::new()),
            store: store,
        }
    }

    fn get_storage(&self) -> Result<Option<Storage>, String> {
        window()
            .unwrap()
            .local_storage()
            .map_err(|e| format!("{:?}", e))
    }

    #[instrument(skip(self))]
    async fn synchronize(&self) -> Result<(), String> {
        info!("Synchronizing Recipes");
        let storage = self.get_storage()?.unwrap();
        let recipes = self
            .store
            .get_recipes()
            .await
            .map_err(|e| format!("{:?}", e))?;
        storage
            .set_item(
                "recipes",
                &(to_string(&recipes).map_err(|e| format!("{:?}", e))?),
            )
            .map_err(|e| format!("{:?}", e))?;
        info!("Synchronizing categories");
        match self.store.get_categories().await {
            Ok(Some(categories_content)) => {
                debug!(categories=?categories_content);
                storage
                    .set_item("categories", &categories_content)
                    .map_err(|e| format!("{:?}", e))?;
            }
            Ok(None) => {
                warn!("There is no category file");
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn fetch_categories_from_storage(
        &self,
    ) -> Result<Option<BTreeMap<String, String>>, String> {
        let storage = self.get_storage()?.unwrap();
        match storage
            .get_item("categories")
            .map_err(|e| format!("{:?}", e))?
        {
            Some(s) => {
                let parsed = from_str::<String>(&s).map_err(|e| format!("{}", e))?;
                match parse::as_categories(&parsed) {
                    Ok(categories) => Ok(Some(categories)),
                    Err(e) => {
                        debug!("Error parsing categories {}", e);
                        Err(format!("Error parsing categories {}", e))
                    }
                }
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    pub fn fetch_recipes_from_storage(
        &self,
    ) -> Result<(Option<Recipe>, Option<Vec<(usize, Recipe)>>), String> {
        let storage = self.get_storage()?.unwrap();
        let mut staples = None;
        match storage
            .get_item("recipes")
            .map_err(|e| format!("{:?}", e))?
        {
            Some(s) => {
                let parsed = from_str::<Vec<String>>(&s).map_err(|e| format!("{}", e))?;
                let mut parsed_list = Vec::new();
                for r in parsed {
                    let recipe = match parse::as_recipe(&r) {
                        Ok(r) => r,
                        Err(e) => {
                            error!("Error parsing recipe {}", e);
                            continue;
                        }
                    };
                    if recipe.title == "Staples" {
                        staples = Some(recipe);
                    } else {
                        parsed_list.push(recipe);
                    }
                }
                Ok((staples, Some(parsed_list.drain(0..).enumerate().collect())))
            }
            None => Ok((None, None)),
        }
    }

    async fn fetch_recipes(
        &self,
    ) -> Result<(Option<Recipe>, Option<Vec<(usize, Recipe)>>), String> {
        Ok(self.fetch_recipes_from_storage()?)
    }

    async fn fetch_categories(&self) -> Result<Option<BTreeMap<String, String>>, String> {
        Ok(self.fetch_categories_from_storage()?)
    }

    #[instrument(skip(self))]
    pub async fn refresh(&mut self) -> Result<(), String> {
        self.synchronize().await?;
        debug!("refreshing recipes");
        if let (staples, Some(r)) = self.fetch_recipes().await? {
            self.set_recipes(r);
            self.staples.set(staples);
        }
        debug!("refreshing categories");
        if let Some(categories) = self.fetch_categories().await? {
            self.set_categories(categories);
        }
        Ok(())
    }

    pub fn get_recipe_by_index(&self, idx: usize) -> Option<Signal<Recipe>> {
        self.recipes.get().get(idx).map(|(_, r)| r.clone())
    }

    #[instrument(skip(self))]
    pub fn get_shopping_list(
        &self,
        show_staples: bool,
    ) -> BTreeMap<String, Vec<(Ingredient, BTreeSet<String>)>> {
        let mut acc = IngredientAccumulator::new();
        let recipe_counts = self.menu_list.get();
        for (idx, count) in recipe_counts.iter() {
            for _ in 0..*count {
                acc.accumulate_from(self.get_recipe_by_index(*idx).unwrap().get().as_ref());
            }
        }
        if show_staples {
            if let Some(staples) = self.staples.get().as_ref() {
                acc.accumulate_from(staples);
            }
        }
        let mut ingredients = acc.ingredients();
        let mut groups = BTreeMap::new();
        let cat_map = self.category_map.get().clone();
        for (_, (i, recipes)) in ingredients.iter_mut() {
            let category = if let Some(cat) = cat_map.get(&i.name) {
                cat.clone()
            } else {
                "other".to_owned()
            };
            i.category = category.clone();
            groups
                .entry(category)
                .or_insert(vec![])
                .push((i.clone(), recipes.clone()));
        }
        debug!(?self.category_map);
        // FIXM(jwall): Sort by categories and names.
        groups
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

    pub fn set_categories(&mut self, categories: BTreeMap<String, String>) {
        self.category_map.set(categories);
    }
}
