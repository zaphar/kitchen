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

use base64;
use reqwasm;
use serde_json::{from_str, to_string};
use sycamore::prelude::*;
use tracing::{debug, error, instrument, warn};

use client_api::*;
use recipes::{parse, IngredientKey, Recipe, RecipeEntry};
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::Storage;

use crate::{app_state::AppState, js_lib};

// FIXME(jwall): We should be able to delete this now.
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

fn token68(user: String, pass: String) -> String {
    base64::encode(format!("{}:{}", user, pass))
}

#[derive(Clone, Debug)]
pub struct LocalStore {
    store: Storage,
}

impl LocalStore {
    pub fn new() -> Self {
        Self {
            store: js_lib::get_storage(),
        }
    }

    /// Gets user data from local storage.
    pub fn get_user_data(&self) -> Option<UserData> {
        self.store
            .get("user_data")
            .map_or(None, |val| val.map(|val| from_str(&val).unwrap_or(None)))
            .flatten()
    }

    // Set's user data to local storage.
    pub fn set_user_data(&self, data: Option<&UserData>) {
        if let Some(data) = data {
            self.store
                .set("user_data", &to_string(data).unwrap_throw())
                .unwrap_throw();
        } else {
            self.store.delete("user_data").unwrap_throw();
        }
    }

    /// Gets categories from local storage.
    pub fn get_categories(&self) -> Option<String> {
        self.store.get("categories").unwrap_throw()
    }

    /// Set the categories to the given string.
    pub fn set_categories(&self, categories: Option<&String>) {
        if let Some(c) = categories {
            self.store.set("categories", c).unwrap_throw();
        } else {
            self.store.delete("categories").unwrap_throw()
        }
    }

    fn get_storage_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for idx in 0..self.store.length().unwrap() {
            keys.push(self.store.key(idx).unwrap_throw().unwrap_throw())
        }
        keys
    }

    fn get_recipe_keys(&self) -> impl Iterator<Item = String> {
        self.get_storage_keys()
            .into_iter()
            .filter(|k| k.starts_with("recipe:"))
    }

    /// Gets all the recipes from local storage.
    pub fn get_recipes(&self) -> Option<Vec<RecipeEntry>> {
        let mut recipe_list = Vec::new();
        for recipe_key in self.get_recipe_keys() {
            if let Some(entry) = self.store.get(&recipe_key).unwrap_throw() {
                match from_str(&entry) {
                    Ok(entry) => {
                        recipe_list.push(entry);
                    }
                    Err(e) => {
                        error!(recipe_key, err = ?e, "Failed to parse recipe entry");
                    }
                }
            }
        }
        if recipe_list.is_empty() {
            return None;
        }
        Some(recipe_list)
    }

    pub fn get_recipe_entry(&self, id: &str) -> Option<RecipeEntry> {
        self.store
            .get(&recipe_key(id))
            .unwrap_throw()
            .map(|entry| from_str(&entry).unwrap_throw())
    }

    /// Sets the set of recipes to the entries passed in. Deletes any recipes not
    /// in the list.
    pub fn set_all_recipes(&self, entries: &Vec<RecipeEntry>) {
        for recipe_key in self.get_recipe_keys() {
            self.store.delete(&recipe_key).unwrap_throw();
        }
        for entry in entries {
            self.set_recipe_entry(entry);
        }
    }

    /// Set recipe entry in local storage.
    pub fn set_recipe_entry(&self, entry: &RecipeEntry) {
        self.store
            .set(
                &recipe_key(entry.recipe_id()),
                &to_string(&entry).unwrap_throw(),
            )
            .unwrap_throw()
    }

    /// Delete recipe entry from local storage.
    pub fn delete_recipe_entry(&self, recipe_key: &str) {
        self.store.delete(recipe_key).unwrap_throw()
    }

    /// Save working plan to local storage.
    pub fn save_plan(&self, plan: &Vec<(String, i32)>) {
        self.store
            .set("plan", &to_string(&plan).unwrap_throw())
            .unwrap_throw();
    }

    pub fn get_plan(&self) -> Option<Vec<(String, i32)>> {
        if let Some(plan) = self.store.get("plan").unwrap_throw() {
            Some(from_str(&plan).unwrap_throw())
        } else {
            None
        }
    }

    pub fn get_inventory_data(
        &self,
    ) -> Option<(
        BTreeSet<IngredientKey>,
        BTreeMap<IngredientKey, String>,
        Vec<(String, String)>,
    )> {
        if let Some(inventory) = self.store.get("inventory").unwrap_throw() {
            return Some(from_str(&inventory).unwrap_throw());
        }
        return None;
    }

    pub fn delete_inventory_data(&self) {
        self.store.delete("inventory").unwrap_throw();
    }

    pub fn set_inventory_data(
        &self,
        inventory: (
            &BTreeSet<IngredientKey>,
            &BTreeMap<IngredientKey, String>,
            &Vec<(String, String)>,
        ),
    ) {
        self.store
            .set("inventory", &to_string(&inventory).unwrap_throw())
            .unwrap_throw();
    }
}

#[derive(Clone, Debug)]
pub struct HttpStore {
    root: String,
    local_store: LocalStore,
}

impl HttpStore {
    pub fn new(root: String) -> Self {
        Self {
            root,
            local_store: LocalStore::new(),
        }
    }

    pub fn v1_path(&self) -> String {
        let mut path = self.root.clone();
        path.push_str("/v1");
        path
    }

    pub fn v2_path(&self) -> String {
        let mut path = self.root.clone();
        path.push_str("/v2");
        path
    }

    pub fn provide_context<S: Into<String>>(cx: Scope, root: S) {
        provide_context(cx, std::rc::Rc::new(Self::new(root.into())));
    }

    pub fn get_from_context(cx: Scope) -> std::rc::Rc<Self> {
        use_context::<std::rc::Rc<Self>>(cx).clone()
    }

    // NOTE(jwall): We do **not** want to record the password in our logs.
    #[instrument(skip_all, fields(?self, user))]
    pub async fn authenticate(&self, user: String, pass: String) -> Option<UserData> {
        debug!("attempting login request against api.");
        let mut path = self.v1_path();
        path.push_str("/auth");
        let result = reqwasm::http::Request::get(&path)
            .header(
                "Authorization",
                format!("Basic {}", token68(user, pass)).as_str(),
            )
            .send()
            .await;
        if let Ok(resp) = &result {
            if resp.status() == 200 {
                let user_data = resp
                    .json::<AccountResponse>()
                    .await
                    .expect("Unparseable authentication response")
                    .as_success();
                return user_data;
            }
            error!(status = resp.status(), "Login was unsuccessful")
        } else {
            error!(err=?result.unwrap_err(), "Failed to send auth request");
        }
        return None;
    }

    //#[instrument]
    pub async fn get_categories(&self) -> Result<Option<String>, Error> {
        let mut path = self.v1_path();
        path.push_str("/categories");
        let resp = match reqwasm::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(reqwasm::Error::JsError(err)) => {
                error!(path, ?err, "Error hitting api");
                return Ok(self.local_store.get_categories());
            }
            Err(err) => {
                return Err(err)?;
            }
        };
        if resp.status() == 404 {
            debug!("Categories returned 404");
            Ok(None)
        } else if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            let resp = resp.json::<CategoryResponse>().await?.as_success().unwrap();
            Ok(Some(resp))
        }
    }

    #[instrument]
    pub async fn get_recipes(&self) -> Result<Option<Vec<RecipeEntry>>, Error> {
        let mut path = self.v1_path();
        path.push_str("/recipes");
        let resp = match reqwasm::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(reqwasm::Error::JsError(err)) => {
                error!(path, ?err, "Error hitting api");
                return Ok(self.local_store.get_recipes());
            }
            Err(err) => {
                return Err(err)?;
            }
        };
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            let entries = resp
                .json::<RecipeEntryResponse>()
                .await
                .map_err(|e| format!("{}", e))?
                .as_success();
            Ok(entries)
        }
    }

    pub async fn get_recipe_text<S: AsRef<str> + std::fmt::Display>(
        &self,
        id: S,
    ) -> Result<Option<RecipeEntry>, Error> {
        let mut path = self.v1_path();
        path.push_str("/recipe/");
        path.push_str(id.as_ref());
        let resp = match reqwasm::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(reqwasm::Error::JsError(err)) => {
                error!(path, ?err, "Error hitting api");
                return Ok(self.local_store.get_recipe_entry(id.as_ref()));
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
            let entry = resp
                .json::<Response<Option<RecipeEntry>>>()
                .await
                .map_err(|e| format!("{}", e))?
                .as_success()
                .unwrap();
            if let Some(ref entry) = entry {
                self.local_store.set_recipe_entry(entry);
            }
            Ok(entry)
        }
    }

    #[instrument(skip(recipes), fields(count=recipes.len()))]
    pub async fn save_recipes(&self, recipes: Vec<RecipeEntry>) -> Result<(), Error> {
        let mut path = self.v1_path();
        path.push_str("/recipes");
        for r in recipes.iter() {
            if r.recipe_id().is_empty() {
                return Err("Recipe Ids can not be empty".into());
            }
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
        let mut path = self.v1_path();
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

    #[instrument(skip_all)]
    pub async fn save_app_state(&self, state: AppState) -> Result<(), Error> {
        let mut plan = Vec::new();
        for (key, count) in state.recipe_counts.iter() {
            plan.push((key.clone(), *count as i32));
        }
        debug!("Saving plan data");
        self.save_plan(plan).await?;
        debug!("Saving inventory data");
        self.save_inventory_data(
            state.filtered_ingredients,
            state.modified_amts,
            state
                .extras
                .iter()
                .cloned()
                .collect::<Vec<(String, String)>>(),
        )
        .await
    }

    pub async fn save_plan(&self, plan: Vec<(String, i32)>) -> Result<(), Error> {
        let mut path = self.v1_path();
        path.push_str("/plan");
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
        let mut path = self.v1_path();
        path.push_str("/plan");
        let resp = reqwasm::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back");
            let plan = resp
                .json::<PlanDataResponse>()
                .await
                .map_err(|e| format!("{}", e))?
                .as_success();
            Ok(plan)
        }
    }

    pub async fn get_inventory_data(
        &self,
    ) -> Result<
        (
            BTreeSet<IngredientKey>,
            BTreeMap<IngredientKey, String>,
            Vec<(String, String)>,
        ),
        Error,
    > {
        let mut path = self.v2_path();
        path.push_str("/inventory");
        let resp = reqwasm::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            let err = Err(format!("Status: {}", resp.status()).into());
            Ok(match self.local_store.get_inventory_data() {
                Some(val) => val,
                None => return err,
            })
        } else {
            debug!("We got a valid response back");
            let InventoryData {
                filtered_ingredients,
                modified_amts,
                extra_items,
            } = resp
                .json::<InventoryResponse>()
                .await
                .map_err(|e| format!("{}", e))?
                .as_success()
                .unwrap();
            Ok((
                filtered_ingredients.into_iter().collect(),
                modified_amts.into_iter().collect(),
                extra_items,
            ))
        }
    }

    #[instrument]
    pub async fn save_inventory_data(
        &self,
        filtered_ingredients: BTreeSet<IngredientKey>,
        modified_amts: BTreeMap<IngredientKey, String>,
        extra_items: Vec<(String, String)>,
    ) -> Result<(), Error> {
        let mut path = self.v2_path();
        path.push_str("/inventory");
        let filtered_ingredients: Vec<IngredientKey> = filtered_ingredients.into_iter().collect();
        let modified_amts: Vec<(IngredientKey, String)> = modified_amts.into_iter().collect();
        debug!("Storing inventory data in cache");
        let serialized_inventory = to_string(&(filtered_ingredients, modified_amts, extra_items))
            .expect("Unable to encode plan as json");
        debug!("Storing inventory data via API");
        let resp = reqwasm::http::Request::post(&path)
            .body(&serialized_inventory)
            .header("content-type", "application/json")
            .send()
            .await?;
        if resp.status() != 200 {
            debug!("Invalid response back");
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            Ok(())
        }
    }
}
