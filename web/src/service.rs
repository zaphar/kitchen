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
use sycamore::{context::use_context, prelude::*};
use tracing::{debug, error, info, instrument, warn};
use web_sys::Storage;

use recipe_store::*;
use recipes::{parse, Ingredient, IngredientAccumulator, Recipe};

use crate::js_lib;

pub fn get_appservice_from_context() -> AppService {
    use_context::<AppService>()
}

#[derive(Clone, Debug)]
pub struct AppService {
    recipes: Signal<BTreeMap<String, Signal<Recipe>>>,
    staples: Signal<Option<Recipe>>,
    category_map: Signal<BTreeMap<String, String>>,
    menu_list: Signal<BTreeMap<String, usize>>,
    store: HttpStore,
}

impl AppService {
    pub fn new(store: HttpStore) -> Self {
        Self {
            recipes: Signal::new(BTreeMap::new()),
            staples: Signal::new(None),
            category_map: Signal::new(BTreeMap::new()),
            menu_list: Signal::new(BTreeMap::new()),
            store: store,
        }
    }

    fn get_storage(&self) -> Result<Option<Storage>, String> {
        js_lib::get_storage().map_err(|e| format!("{:?}", e))
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

    pub fn get_category_text(&self) -> Result<Option<String>, String> {
        let storage = self.get_storage()?.unwrap();
        storage
            .get_item("categories")
            .map_err(|e| format!("{:?}", e))
    }

    #[instrument(skip(self))]
    pub fn fetch_categories_from_storage(
        &self,
    ) -> Result<Option<BTreeMap<String, String>>, String> {
        match self.get_category_text()? {
            Some(s) => {
                let parsed = from_str::<String>(&s).map_err(|e| format!("{}", e))?;
                if parsed.is_empty() {
                    return Ok(None);
                }
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

    async fn fetch_recipes(
        &self,
    ) -> Result<(Option<Recipe>, Option<BTreeMap<String, Recipe>>), String> {
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

    pub fn get_recipe_by_index(&self, idx: &str) -> Option<Signal<Recipe>> {
        self.recipes.get().get(idx).map(|r| r.clone())
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
                acc.accumulate_from(self.get_recipe_by_index(idx).unwrap().get().as_ref());
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

    pub fn set_recipe_count_by_index(&mut self, i: String, count: usize) {
        let mut v = (*self.menu_list.get()).clone();
        v.insert(i, count);
        self.menu_list.set(v);
    }

    pub fn get_recipe_count_by_index(&self, i: &str) -> usize {
        self.menu_list.get().get(i).map(|i| *i).unwrap_or_default()
    }

    pub fn get_recipes(&self) -> Signal<BTreeMap<String, Signal<Recipe>>> {
        self.recipes.clone()
    }

    pub fn get_menu_list(&self) -> Vec<(String, usize)> {
        self.menu_list
            .get()
            .iter()
            // We exclude recipes in the menu_list with count 0
            .filter(|&(_, count)| *count != 0)
            .map(|(idx, count)| (idx.clone(), *count))
            .collect()
    }

    pub async fn save_recipes(&self, recipes: Vec<RecipeEntry>) -> Result<(), String> {
        self.store.save_recipes(recipes).await?;
        Ok(())
    }

    pub async fn save_categories(&self, categories: String) -> Result<(), String> {
        self.store.save_categories(categories).await?;
        Ok(())
    }

    pub fn set_recipes(&mut self, recipes: BTreeMap<String, Recipe>) {
        self.recipes.set(
            recipes
                .iter()
                .map(|(i, r)| (i.clone(), Signal::new(r.clone())))
                .collect(),
        );
    }

    pub fn set_categories(&mut self, categories: BTreeMap<String, String>) {
        self.category_map.set(categories);
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
