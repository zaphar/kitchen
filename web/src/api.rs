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
use serde_json::{from_str, to_string};
use sycamore::prelude::*;
use tracing::{debug, error, info, instrument, warn};

use recipes::{parse, IngredientKey, Recipe, RecipeEntry};
use wasm_bindgen::JsValue;

use crate::{app_state, js_lib};

#[instrument]
fn filter_recipes(
    recipe_entries: &Option<Vec<RecipeEntry>>,
) -> Result<(Option<Recipe>, Option<BTreeMap<String, Recipe>>), String> {
    match recipe_entries {
        Some(parsed) => {
            let mut staples = None;
            let mut parsed_map = BTreeMap::new();
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

#[instrument(skip(state))]
pub async fn init_page_state(store: &HttpStore, state: &app_state::State) -> Result<(), String> {
    info!("Synchronizing Recipes");
    // TODO(jwall): Make our caching logic using storage more robust.
    let recipes = store.get_recipes().await.map_err(|e| format!("{:?}", e))?;
    if let Ok((staples, recipes)) = filter_recipes(&recipes) {
        state.staples.set(staples);
        if let Some(recipes) = recipes {
            state.recipes.set(recipes);
        }
    }

    if let Ok(Some(plan)) = store.get_plan().await {
        // set the counts.
        for (id, count) in plan {
            state.set_recipe_count_by_index(&id, count as usize);
        }
    } else {
        // Initialize things to zero
        if let Some(rs) = recipes {
            for r in rs {
                if !state.recipe_counts.get().contains_key(r.recipe_id()) {
                    state.set_recipe_count_by_index(&r.recipe_id().to_owned(), 0);
                }
            }
        }
    }
    info!("Synchronizing categories");
    match store.get_categories().await {
        Ok(Some(categories_content)) => {
            debug!(categories=?categories_content);
            let category_map = recipes::parse::as_categories(&categories_content)?;
            state.category_map.set(category_map);
        }
        Ok(None) => {
            warn!("There is no category file");
        }
        Err(e) => {
            error!("{:?}", e);
        }
    }
    info!("Synchronizing inventory data");
    match store.get_inventory_data().await {
        Ok((filtered_ingredients, modified_amts)) => {
            state.reset_modified_amts(modified_amts);
            state.filtered_ingredients.set(filtered_ingredients);
        }
        Err(e) => {
            error!("{:?}", e);
        }
    }
    Ok(())
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

impl From<JsValue> for Error {
    fn from(item: JsValue) -> Self {
        Error(format!("{:?}", item))
    }
}

impl From<String> for Error {
    fn from(item: String) -> Self {
        Error(item)
    }
}

impl From<&'static str> for Error {
    fn from(item: &'static str) -> Self {
        Error(item.to_owned())
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

fn recipe_key<S: std::fmt::Display>(id: S) -> String {
    format!("recipe:{}", id)
}

#[derive(Clone, Debug)]
pub struct HttpStore {
    root: String,
}

impl HttpStore {
    pub fn new(root: String) -> Self {
        Self { root }
    }

    pub fn provide_context<S: Into<String>>(cx: Scope, root: S) {
        provide_context(cx, std::rc::Rc::new(Self::new(root.into())));
    }

    pub fn get_from_context(cx: Scope) -> std::rc::Rc<Self> {
        use_context::<std::rc::Rc<Self>>(cx).clone()
    }

    //#[instrument]
    pub async fn get_categories(&self) -> Result<Option<String>, Error> {
        let mut path = self.root.clone();
        path.push_str("/categories");
        let storage = js_lib::get_storage();
        let resp = match reqwasm::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(reqwasm::Error::JsError(err)) => {
                error!(path, ?err, "Error hitting api");
                return Ok(storage.get("categories")?);
            }
            Err(err) => {
                return Err(err)?;
            }
        };
        if resp.status() == 404 {
            debug!("Categories returned 404");
            storage.remove_item("categories")?;
            Ok(None)
        } else if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            let resp: String = resp.json().await?;
            storage.set("categories", &resp)?;
            Ok(Some(resp))
        }
    }

    #[instrument]
    pub async fn get_recipes(&self) -> Result<Option<Vec<RecipeEntry>>, Error> {
        let mut path = self.root.clone();
        path.push_str("/recipes");
        let storage = js_lib::get_storage();
        let resp = match reqwasm::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(reqwasm::Error::JsError(err)) => {
                error!(path, ?err, "Error hitting api");
                let mut entries = Vec::new();
                for key in js_lib::get_storage_keys() {
                    if key.starts_with("recipe:") {
                        let entry = from_str(&storage.get_item(&key)?.unwrap())
                            .map_err(|e| format!("{}", e))?;
                        entries.push(entry);
                    }
                }
                return Ok(Some(entries));
            }
            Err(err) => {
                return Err(err)?;
            }
        };
        let storage = js_lib::get_storage();
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            let entries: Option<Vec<RecipeEntry>> =
                resp.json().await.map_err(|e| format!("{}", e))?;
            if let Some(ref entries) = entries {
                for r in entries.iter() {
                    storage.set(
                        &recipe_key(r.recipe_id()),
                        &to_string(&r).expect("Unable to serialize recipe entries"),
                    )?;
                }
            }
            Ok(entries)
        }
    }

    pub async fn get_recipe_text<S: AsRef<str> + std::fmt::Display>(
        &self,
        id: S,
    ) -> Result<Option<RecipeEntry>, Error> {
        let mut path = self.root.clone();
        path.push_str("/recipe/");
        path.push_str(id.as_ref());
        let storage = js_lib::get_storage();
        let resp = match reqwasm::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(reqwasm::Error::JsError(err)) => {
                error!(path, ?err, "Error hitting api");
                return match storage.get(&recipe_key(&id))? {
                    Some(s) => Ok(Some(from_str(&s).map_err(|e| format!("{}", e))?)),
                    None => Ok(None),
                };
            }
            Err(err) => {
                return Err(err)?;
            }
        };
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else if resp.status() == 404 {
            debug!("Recipe doesn't exist");
            Ok(None)
        } else {
            debug!("We got a valid response back!");
            let entry: Option<RecipeEntry> = resp.json().await.map_err(|e| format!("{}", e))?;
            if let Some(ref entry) = entry {
                let serialized: String = to_string(entry).map_err(|e| format!("{}", e))?;
                storage.set(&recipe_key(entry.recipe_id()), &serialized)?
            }
            Ok(entry)
        }
    }

    #[instrument(skip(recipes), fields(count=recipes.len()))]
    pub async fn save_recipes(&self, recipes: Vec<RecipeEntry>) -> Result<(), Error> {
        let mut path = self.root.clone();
        path.push_str("/recipes");
        let storage = js_lib::get_storage();
        for r in recipes.iter() {
            if r.recipe_id().is_empty() {
                return Err("Recipe Ids can not be empty".into());
            }
            storage.set(
                &recipe_key(r.recipe_id()),
                &to_string(&r).expect("Unable to serialize recipe entries"),
            )?;
        }
        let serialized = to_string(&recipes).expect("Unable to serialize recipe entries");
        let resp = reqwasm::http::Request::post(&path)
            .body(&serialized)
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
    pub async fn save_categories(&self, categories: String) -> Result<(), Error> {
        let mut path = self.root.clone();
        path.push_str("/categories");
        let storage = js_lib::get_storage();
        storage.set("categories", &categories)?;
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

    pub async fn save_plan(&self, plan: Vec<(String, i32)>) -> Result<(), Error> {
        let mut path = self.root.clone();
        path.push_str("/plan");
        let storage = js_lib::get_storage();
        let serialized_plan = to_string(&plan).expect("Unable to encode plan as json");
        storage.set("plan", &serialized_plan)?;
        let resp = reqwasm::http::Request::post(&path)
            .body(to_string(&plan).expect("Unable to encode plan as json"))
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

    pub async fn get_plan(&self) -> Result<Option<Vec<(String, i32)>>, Error> {
        let mut path = self.root.clone();
        path.push_str("/plan");
        let resp = reqwasm::http::Request::get(&path).send().await?;
        let storage = js_lib::get_storage();
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back");
            let plan: Option<Vec<(String, i32)>> =
                resp.json().await.map_err(|e| format!("{}", e))?;
            if let Some(ref entry) = plan {
                let serialized: String = to_string(entry).map_err(|e| format!("{}", e))?;
                storage.set("plan", &serialized)?
            }
            Ok(plan)
        }
    }

    pub async fn get_inventory_data(
        &self,
    ) -> Result<(BTreeSet<IngredientKey>, BTreeMap<IngredientKey, String>), Error> {
        let mut path = self.root.clone();
        path.push_str("/inventory");
        let storage = js_lib::get_storage();
        let resp = reqwasm::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            let err = Err(format!("Status: {}", resp.status()).into());
            Ok(match storage.get("inventory") {
                Ok(Some(val)) => from_str(&val).expect("Failed to deserialize cached val"),
                Ok(None) | Err(_) => return err,
            })
        } else {
            debug!("We got a valid response back");
            let (filtered_ingredients, modified_amts): (
                Vec<IngredientKey>,
                Vec<(IngredientKey, String)>,
            ) = resp.json().await.map_err(|e| format!("{}", e))?;
            let _ = storage.set(
                "inventory",
                &to_string(&(&filtered_ingredients, &modified_amts))
                    .expect("Failed to serialize inventory data"),
            );
            Ok((
                filtered_ingredients.into_iter().collect(),
                modified_amts.into_iter().collect(),
            ))
        }
    }

    pub async fn save_inventory_data(
        &self,
        filtered_ingredients: BTreeSet<IngredientKey>,
        modified_amts: BTreeMap<IngredientKey, String>,
    ) -> Result<(), Error> {
        let mut path = self.root.clone();
        let filtered_ingredients: Vec<IngredientKey> = filtered_ingredients.into_iter().collect();
        let modified_amts: Vec<(IngredientKey, String)> = modified_amts.into_iter().collect();
        let serialized_inventory = to_string(&(filtered_ingredients, modified_amts))
            .expect("Unable to encode plan as json");
        let storage = js_lib::get_storage();
        let _ = storage.set("inventory", &serialized_inventory);
        path.push_str("/inventory");
        let resp = reqwasm::http::Request::post(&path)
            .body(&serialized_inventory)
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
