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

use reqwasm;
//use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use sycamore::prelude::*;
use tracing::{debug, error, info, instrument, warn};
use web_sys::Storage;

use recipe_store::*;
use recipes::{parse, Ingredient, IngredientAccumulator, Recipe};

use crate::js_lib;

pub fn get_appservice_from_context(cx: Scope) -> &AppService {
    use_context::<AppService>(cx)
}

// TODO(jwall): We should not be cloning this.
#[derive(Clone, Debug)]
pub struct AppService {
    recipe_counts: RcSignal<BTreeMap<String, usize>>,
    staples: RcSignal<Option<Recipe>>,
    recipes: RcSignal<BTreeMap<String, Recipe>>,
    category_map: RcSignal<BTreeMap<String, String>>,
    store: HttpStore,
}

impl AppService {
    pub fn new(store: HttpStore) -> Self {
        Self {
            recipe_counts: create_rc_signal(BTreeMap::new()),
            staples: create_rc_signal(None),
            recipes: create_rc_signal(BTreeMap::new()),
            category_map: create_rc_signal(BTreeMap::new()),
            store: store,
        }
    }

    fn get_storage(&self) -> Result<Option<Storage>, String> {
        js_lib::get_storage().map_err(|e| format!("{:?}", e))
    }

    pub fn get_menu_list(&self) -> Vec<(String, usize)> {
        self.recipe_counts
            .get()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    #[instrument(skip(self))]
    pub async fn synchronize(&mut self) -> Result<(), String> {
        info!("Synchronizing Recipes");
        // TODO(jwall): Make our caching logic using storage more robust.
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
        if let Ok((staples, recipes)) = self.fetch_recipes_from_storage() {
            self.staples.set(staples);
            if let Some(recipes) = recipes {
                self.recipes.set(recipes);
            }
        }
        if let Some(rs) = recipes {
            for r in rs {
                if !self.recipe_counts.get().contains_key(r.recipe_id()) {
                    self.set_recipe_count_by_index(&r.recipe_id().to_owned(), 0);
                }
            }
        }
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

    pub fn get_recipe_count_by_index(&self, key: &String) -> Option<usize> {
        self.recipe_counts.get().get(key).cloned()
    }

    pub fn set_recipe_count_by_index(&mut self, key: &String, count: usize) -> usize {
        let mut counts = self.recipe_counts.get().as_ref().clone();
        counts.insert(key.clone(), count);
        self.recipe_counts.set(counts);
        count
    }

    #[instrument(skip(self))]
    pub fn get_shopping_list(
        &self,
        show_staples: bool,
    ) -> BTreeMap<String, Vec<(Ingredient, BTreeSet<String>)>> {
        let mut acc = IngredientAccumulator::new();
        let recipe_counts = self.get_menu_list();
        for (idx, count) in recipe_counts.iter() {
            for _ in 0..*count {
                acc.accumulate_from(self.recipes.get().get(idx).unwrap());
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
        // FIXME(jwall): Sort by categories and names.
        groups
    }

    pub fn get_category_text(&self) -> Result<Option<String>, String> {
        let storage = self.get_storage()?.unwrap();
        storage
            .get_item("categories")
            .map_err(|e| format!("{:?}", e))
    }

    #[instrument(skip(self))]
    pub fn fetch_recipes_from_storage(
        &self,
    ) -> Result<(Option<Recipe>, Option<BTreeMap<String, Recipe>>), String> {
        let storage = self.get_storage()?.unwrap();
        let mut staples = None;
        match storage
            .get_item("recipes")
            .map_err(|e| format!("{:?}", e))?
        {
            Some(s) => {
                let parsed = from_str::<Vec<RecipeEntry>>(&s).map_err(|e| format!("{}", e))?;
                let mut parsed_map = BTreeMap::new();
                // TODO(jwall): Utilize the id instead of the index from now on.
                for r in parsed {
                    let recipe = match parse::as_recipe(&r.recipe_text()) {
                        Ok(r) => r,
                        Err(e) => {
                            error!("Error parsing recipe {}", e);
                            continue;
                        }
                    };
                    if recipe.title == "Staples" {
                        staples = Some(recipe);
                    } else {
                        parsed_map.insert(r.recipe_id().to_owned(), recipe);
                    }
                }
                Ok((staples, Some(parsed_map)))
            }
            None => Ok((None, None)),
        }
    }

    pub fn fetch_recipe_text(&self, id: &str) -> Result<Option<RecipeEntry>, String> {
        let storage = self.get_storage()?.unwrap();
        if let Some(s) = storage
            .get_item("recipes")
            .map_err(|e| format!("{:?}", e))?
        {
            let parsed = from_str::<Vec<RecipeEntry>>(&s).map_err(|e| format!("{}", e))?;
            for r in parsed {
                if r.recipe_id() == id {
                    return Ok(Some(r));
                }
            }
        }
        return Ok(None);
    }

    pub async fn save_recipes(&self, recipes: Vec<RecipeEntry>) -> Result<(), String> {
        self.store.save_recipes(recipes).await?;
        Ok(())
    }

    pub async fn save_categories(&self, categories: String) -> Result<(), String> {
        self.store.save_categories(categories).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Error(String);

impl From<std::io::Error> for Error {
    fn from(item: std::io::Error) -> Self {
        Error(format!("{:?}", item))
    }
}

impl From<Error> for String {
    fn from(item: Error) -> Self {
        format!("{:?}", item)
    }
}

impl From<String> for Error {
    fn from(item: String) -> Self {
        Error(item)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(item: std::string::FromUtf8Error) -> Self {
        Error(format!("{:?}", item))
    }
}

impl From<reqwasm::Error> for Error {
    fn from(item: reqwasm::Error) -> Self {
        Error(format!("{:?}", item))
    }
}

#[derive(Clone, Debug)]
pub struct HttpStore {
    root: String,
}

impl HttpStore {
    pub fn new(root: String) -> Self {
        Self { root }
    }

    #[instrument]
    async fn get_categories(&self) -> Result<Option<String>, Error> {
        let mut path = self.root.clone();
        path.push_str("/categories");
        let resp = reqwasm::http::Request::get(&path).send().await?;
        if resp.status() == 404 {
            debug!("Categories returned 404");
            Ok(None)
        } else if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            let resp = resp.text().await;
            Ok(Some(resp.map_err(|e| format!("{}", e))?))
        }
    }

    #[instrument]
    async fn get_recipes(&self) -> Result<Option<Vec<RecipeEntry>>, Error> {
        let mut path = self.root.clone();
        path.push_str("/recipes");
        let resp = reqwasm::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            Ok(resp.json().await.map_err(|e| format!("{}", e))?)
        }
    }

    #[instrument(skip(recipes), fields(count=recipes.len()))]
    async fn save_recipes(&self, recipes: Vec<RecipeEntry>) -> Result<(), Error> {
        let mut path = self.root.clone();
        path.push_str("/recipes");
        let resp = reqwasm::http::Request::post(&path)
            .body(to_string(&recipes).expect("Unable to serialize recipe entries"))
            .header("content-type", "application/json")
            .send()
            .await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            Ok(())
        }
    }

    #[instrument(skip(categories))]
    async fn save_categories(&self, categories: String) -> Result<(), Error> {
        let mut path = self.root.clone();
        path.push_str("/categories");
        let resp = reqwasm::http::Request::post(&path)
            .body(to_string(&categories).expect("Unable to encode categories as json"))
            .header("content-type", "application/json")
            .send()
            .await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            Ok(())
        }
    }
}
