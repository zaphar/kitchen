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

use base64::{self, Engine};
use chrono::NaiveDate;
use gloo_net;
// TODO(jwall): Remove this when we have gone a few migrations past.
use serde_json::{from_str, to_string};
use sycamore::prelude::*;
use tracing::{debug, error, field::debug, instrument};

use anyhow::Result;
use client_api::*;
use recipes::{IngredientKey, RecipeEntry};
use serde_wasm_bindgen::{from_value, Serializer};
use wasm_bindgen::JsValue;
// TODO(jwall): Remove this when we have gone a few migrations past.
use web_sys::{window, Storage};

fn to_js<T: serde::ser::Serialize>(value: T) -> Result<JsValue, serde_wasm_bindgen::Error>
{
    let s = Serializer::new().serialize_maps_as_objects(true);
    value.serialize(&s)
}

use crate::{
    app_state::{parse_recipes, AppState},
    js_lib::{self, DBFactory},
};

#[allow(dead_code)]
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

impl From<gloo_net::Error> for Error {
    fn from(item: gloo_net::Error) -> Self {
        Error(format!("{:?}", item))
    }
}

fn token68(user: String, pass: String) -> String {
    base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", user, pass))
}

fn convert_to_io_error<V, E>(res: Result<V, E>) -> Result<V, std::io::Error>
where
    E: Into<Box<dyn std::error::Error>> + std::fmt::Debug,
{
    match res {
        Ok(v) => Ok(v),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e))),
    }
}

#[derive(Clone, Debug)]
pub struct LocalStore {
    // TODO(zaphar): Remove this when it's safe to delete the migration
    old_store: Storage,
    store: DBFactory<'static>,
}

const APP_STATE_KEY: &'static str = "app-state";
const USER_DATA_KEY: &'static str = "user_data";

impl LocalStore {
    pub fn new() -> Self {
        Self {
            store: DBFactory::default(),
            old_store: js_lib::get_storage(),
        }
    }

    pub async fn migrate(&self) {
        // 1. migrate app-state from localstore to indexeddb
        debug!("Peforming localstorage migration");
        if let Ok(Some(v)) = self.old_store.get("app_state") {
            if let Ok(Some(local_state)) = from_str::<Option<AppState>>(&v) {
                self.store_app_state(&local_state).await;
            }
        }
        let _ = self.old_store.remove_item("app_state");
        // 2. migrate user-state from localstore to indexeddb
        if let Ok(Some(v)) = self.old_store.get(USER_DATA_KEY) {
            if let Ok(local_user_data) = from_str::<Option<UserData>>(&v) {
                self.set_user_data(local_user_data.as_ref()).await;
            }
        }
        let _ = self.old_store.remove_item(USER_DATA_KEY);
        // 3. Recipes
        let store_len = self.old_store.length().unwrap();
        let mut key_list = Vec::new();
        for i in 0..store_len {
            let key = self.old_store.key(i).unwrap().unwrap();
            if key.starts_with("recipe:") {
                key_list.push(key);
            }
        }
        for k in key_list {
            if let Ok(Some(recipe)) = self.old_store.get(&k) {
                if let Ok(recipe) = from_str::<RecipeEntry>(&recipe) {
                    self.set_recipe_entry(&recipe).await;
                }
            }
            let _ = self.old_store.delete(&k);
        }
    }

    #[instrument(skip_all)]
    pub async fn store_app_state(&self, state: &AppState) {
        let state = match to_js(state) {
            Ok(state) => state,
            Err(err) => {
                error!(?err, ?state, "Error deserializing app_state");
                return;
            }
        };
        web_sys::console::log_1(&state);
        let key = to_js(APP_STATE_KEY).expect("Failed to serialize key");
        self.store
            .rw_transaction(&[js_lib::STATE_STORE_NAME], |trx| async move {
                let object_store = trx.object_store(js_lib::STATE_STORE_NAME)?;
                object_store.put_kv(&key, &state).await?;
                Ok(())
            })
            .await
            .expect("Failed to store app-state");
    }

    #[instrument]
    pub async fn fetch_app_state(&self) -> Option<AppState> {
        debug!("Loading state from local store");
        let recipes = parse_recipes(&self.get_recipes().await).expect("Failed to parse recipes");
        self.store
            .ro_transaction(&[js_lib::STATE_STORE_NAME], |trx| async move {
                let key = convert_to_io_error(to_js(APP_STATE_KEY))?;
                let object_store = trx.object_store(js_lib::STATE_STORE_NAME)?;
                let mut app_state: AppState = match object_store.get(&key).await? {
                    Some(s) => convert_to_io_error(from_value(s))?,
                    None => return Ok(None),
                };

                if let Some(recipes) = recipes {
                    debug!("Populating recipes");
                    for (id, recipe) in recipes {
                        debug!(id, "Adding recipe from local storage");
                        app_state.recipes.insert(id, recipe);
                    }
                }
                Ok(Some(app_state))
            })
            .await
            .expect("Failed to fetch app-state")
    }

    #[instrument]
    /// Gets user data from local storage.
    pub async fn get_user_data(&self) -> Option<UserData> {
        self.store
            .ro_transaction(&[js_lib::STATE_STORE_NAME], |trx| async move {
                let key = to_js(USER_DATA_KEY).expect("Failed to serialize key");
                let object_store = trx
                    .object_store(js_lib::STATE_STORE_NAME)?;
                let user_data: UserData = match object_store
                    .get(&key)
                    .await?
                {
                    Some(s) => convert_to_io_error(from_value(s))?,
                    None => return Ok(None),
                };
                Ok(Some(user_data))
            })
            .await
            .expect("Failed to fetch user_data")
    }

    #[instrument]
    // Set's user data to local storage.
    pub async fn set_user_data(&self, data: Option<&UserData>) {
        let key = to_js(USER_DATA_KEY).expect("Failed to serialize key");
        if let Some(data) = data {
            let data = data.clone();
            self.store
                .rw_transaction(&[js_lib::STATE_STORE_NAME], |trx| async move {
                    let object_store = trx
                        .object_store(js_lib::STATE_STORE_NAME)?;
                    object_store
                        .put_kv(
                            &key,
                            &convert_to_io_error(to_js(&data))?,
                        )
                        .await?;
                    Ok(())
                })
                .await
                .expect("Failed to set user_data");
        } else {
            self.store
                .rw_transaction(&[js_lib::STATE_STORE_NAME], |trx| async move {
                    let object_store = trx
                        .object_store(js_lib::STATE_STORE_NAME)?;
                    object_store
                        .delete(&key)
                        .await?;
                    Ok(())
                })
                .await
                .expect("Failed to delete user_data");
        }
    }

    #[instrument]
    async fn get_recipe_keys(&self) -> impl Iterator<Item = String> {
        self.store
            .ro_transaction(&[js_lib::RECIPE_STORE_NAME], |trx| async move {
                let mut keys = Vec::new();
                let object_store = trx
                    .object_store(js_lib::RECIPE_STORE_NAME)?;
                let key_vec = object_store
                    .get_all_keys(None)
                    .await?;
                for k in key_vec {
                    if let Ok(v) = from_value(k) {
                        keys.push(v);
                    }
                }
                Ok(keys)
            })
            .await
            .expect("Failed to get storage keys")
            .into_iter()
    }

    #[instrument]
    /// Gets all the recipes from local storage.
    pub async fn get_recipes(&self) -> Option<Vec<RecipeEntry>> {
        self.store
            .ro_transaction(&[js_lib::RECIPE_STORE_NAME], |trx| async move {
                let mut recipe_list = Vec::new();
                let object_store = trx
                    .object_store(js_lib::RECIPE_STORE_NAME)?;
                let mut c = object_store
                    .cursor()
                    .open()
                    .await?;
                while let Some(value) = c.value() {
                    recipe_list.push(convert_to_io_error(from_value(value))?);
                    c.advance(1)
                        .await?;
                }
                if recipe_list.is_empty() {
                    return Ok(None);
                }
                Ok(Some(recipe_list))
            })
            .await
            .expect("Failed to get recipes")
    }

    #[instrument]
    pub async fn get_recipe_entry(&self, id: &str) -> Option<RecipeEntry> {
        let key = to_js(id).expect("Failed to serialize key");
        self.store
            .ro_transaction(&[js_lib::RECIPE_STORE_NAME], |trx| async move {
                let object_store = trx
                    .object_store(js_lib::RECIPE_STORE_NAME)?;
                let entry: Option<RecipeEntry> = match object_store
                    .get(&key)
                    .await?
                {
                    Some(v) => convert_to_io_error(from_value(v))?,
                    None => None,
                };
                Ok(entry)
            })
            .await
            .expect("Failed to get recipes")
    }

    #[instrument]
    /// Sets the set of recipes to the entries passed in. Deletes any recipes not
    /// in the list.
    pub async fn set_all_recipes(&self, entries: &Vec<RecipeEntry>) {
        for recipe_key in self.get_recipe_keys().await {
            let key = to_js(&recipe_key).expect("Failed to serialize key");
            self.store
                .rw_transaction(&[js_lib::STATE_STORE_NAME], |trx| async move {
                    let object_store = trx
                        .object_store(js_lib::STATE_STORE_NAME)?;
                    object_store
                        .delete(&key)
                        .await?;
                    Ok(())
                })
                .await
                .expect("Failed to delete user_data");
        }
        for entry in entries {
            let entry = entry.clone();
            let key = to_js(entry.recipe_id()).expect("Failed to serialize recipe key");
            self.store
                .rw_transaction(&[js_lib::RECIPE_STORE_NAME], |trx| async move {
                    let object_store = trx
                        .object_store(js_lib::RECIPE_STORE_NAME)?;
                    object_store
                        .put_kv(
                            &key,
                           &convert_to_io_error(to_js(&entry))?,
                        )
                        .await?;
                    Ok(())
                })
                .await
                .expect("Failed to store recipe entry");
        }
    }

    #[instrument]
    /// Set recipe entry in local storage.
    pub async fn set_recipe_entry(&self, entry: &RecipeEntry) {
        let entry = entry.clone();
        let key = to_js(entry.recipe_id()).expect("Failed to serialize recipe key");
        self.store
            .rw_transaction(&[js_lib::RECIPE_STORE_NAME], |trx| async move {
                let object_store = trx
                    .object_store(js_lib::RECIPE_STORE_NAME)?;
                object_store
                    .put_kv(
                        &key,
                        &convert_to_io_error(to_js(&entry))?,
                    )
                    .await?;
                Ok(())
            })
            .await
            .expect("Failed to store recipe entry");
    }

    #[instrument]
    /// Delete recipe entry from local storage.
    pub async fn delete_recipe_entry(&self, recipe_id: &str) {
        let key = to_js(recipe_id).expect("Failed to serialize key");
        self.store
            .rw_transaction(&[js_lib::RECIPE_STORE_NAME], |trx| async move {
                let object_store = trx
                    .object_store(js_lib::RECIPE_STORE_NAME)?;
                object_store
                    .delete(&key)
                    .await?;
                Ok(())
            })
            .await
            .expect("Failed to delete user_data");
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
        let mut path = self.v2_path();
        path.push_str("/auth");
        let request = gloo_net::http::Request::get(&path)
            .header(
                "authorization",
                format!("Basic {}", token68(user, pass)).as_str(),
            )
            .mode(web_sys::RequestMode::SameOrigin)
            .credentials(web_sys::RequestCredentials::SameOrigin)
            .build()
            .expect("Failed to build request");
        debug!(?request, "Sending auth request");
        let result = request.send().await;
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

    #[instrument]
    pub async fn fetch_user_data(&self) -> Option<UserData> {
        debug!("Retrieving User Account data");
        let mut path = self.v2_path();
        path.push_str("/account");
        let result = gloo_net::http::Request::get(&path).send().await;
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
    pub async fn fetch_categories(&self) -> Result<Option<Vec<(String, String)>>, Error> {
        let mut path = self.v2_path();
        path.push_str("/category_map");
        let resp = match gloo_net::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(gloo_net::Error::JsError(err)) => {
                error!(path, ?err, "Error hitting api");
                return Ok(None);
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
            let resp = resp
                .json::<CategoryMappingResponse>()
                .await?
                .as_success()
                .unwrap();
            Ok(Some(resp))
        }
    }

    #[instrument]
    pub async fn fetch_recipes(&self) -> Result<Option<Vec<RecipeEntry>>, Error> {
        let mut path = self.v2_path();
        path.push_str("/recipes");
        let resp = match gloo_net::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(gloo_net::Error::JsError(err)) => {
                error!(path, ?err, "Error hitting api");
                return Ok(self.local_store.get_recipes().await);
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

    pub async fn fetch_recipe_text<S: AsRef<str> + std::fmt::Display>(
        &self,
        id: S,
    ) -> Result<Option<RecipeEntry>, Error> {
        let mut path = self.v2_path();
        path.push_str("/recipe/");
        path.push_str(id.as_ref());
        let resp = match gloo_net::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(gloo_net::Error::JsError(err)) => {
                error!(path, ?err, "Error hitting api");
                return Ok(self.local_store.get_recipe_entry(id.as_ref()).await);
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
                self.local_store.set_recipe_entry(entry).await;
            }
            Ok(entry)
        }
    }

    #[instrument]
    pub async fn delete_recipe<S>(&self, recipe: S) -> Result<(), Error>
    where
        S: AsRef<str> + std::fmt::Debug,
    {
        let mut path = self.v2_path();
        path.push_str("/recipe");
        path.push_str(&format!("/{}", recipe.as_ref()));
        let resp = gloo_net::http::Request::delete(&path).send().await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            Ok(())
        }
    }

    #[instrument(skip(recipes), fields(count=recipes.len()))]
    pub async fn store_recipes(&self, recipes: Vec<RecipeEntry>) -> Result<(), Error> {
        let mut path = self.v2_path();
        path.push_str("/recipes");
        for r in recipes.iter() {
            if r.recipe_id().is_empty() {
                return Err("Recipe Ids can not be empty".into());
            }
        }
        let resp = gloo_net::http::Request::post(&path)
            .json(&recipes)
            .expect("Failed to set body")
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
    pub async fn store_categories(&self, categories: &Vec<(String, String)>) -> Result<(), Error> {
        let mut path = self.v2_path();
        path.push_str("/category_map");
        let resp = gloo_net::http::Request::post(&path)
            .json(&categories)
            .expect("Failed to set body")
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
    pub async fn store_app_state(&self, state: &AppState) -> Result<(), Error> {
        let mut plan = Vec::new();
        for (key, count) in state.recipe_counts.iter() {
            plan.push((key.clone(), *count as i32));
        }
        if let Some(cached_plan_date) = &state.selected_plan_date {
            debug!(?plan, "Saving plan data");
            self.store_plan_for_date(plan, cached_plan_date).await?;
            debug!("Saving inventory data");
            self.store_inventory_data_for_date(
                state.filtered_ingredients.clone(),
                state.modified_amts.clone(),
                state
                    .extras
                    .iter()
                    .cloned()
                    .collect::<Vec<(String, String)>>(),
                cached_plan_date,
            )
            .await
        } else {
            debug!("Saving plan data");
            self.store_plan(plan).await?;
            debug!("Saving inventory data");
            self.store_inventory_data(
                state.filtered_ingredients.clone(),
                state.modified_amts.clone(),
                state
                    .extras
                    .iter()
                    .cloned()
                    .collect::<Vec<(String, String)>>(),
            )
            .await
        }
    }

    pub async fn store_plan(&self, plan: Vec<(String, i32)>) -> Result<(), Error> {
        let mut path = self.v2_path();
        path.push_str("/plan");
        let resp = gloo_net::http::Request::post(&path)
            .json(&plan)
            .expect("Failed to set body")
            .send()
            .await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            Ok(())
        }
    }

    pub async fn store_plan_for_date(
        &self,
        plan: Vec<(String, i32)>,
        date: &NaiveDate,
    ) -> Result<(), Error> {
        let mut path = self.v2_path();
        path.push_str("/plan");
        path.push_str("/at");
        path.push_str(&format!("/{}", date));
        let resp = gloo_net::http::Request::post(&path)
            .json(&plan)
            .expect("Failed to set body")
            .send()
            .await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            Ok(())
        }
    }

    pub async fn fetch_plan_dates(&self) -> Result<Option<Vec<NaiveDate>>, Error> {
        let mut path = self.v2_path();
        path.push_str("/plan");
        path.push_str("/all");
        let resp = gloo_net::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back");
            let plan = resp
                .json::<Response<Vec<NaiveDate>>>()
                .await
                .map_err(|e| format!("{}", e))?
                .as_success();
            Ok(plan)
        }
    }

    pub async fn delete_plan_for_date(&self, date: &NaiveDate) -> Result<(), Error> {
        let mut path = self.v2_path();
        path.push_str("/plan");
        path.push_str("/at");
        path.push_str(&format!("/{}", date));
        let resp = gloo_net::http::Request::delete(&path).send().await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            Ok(())
        }
    }

    pub async fn fetch_plan_for_date(
        &self,
        date: &NaiveDate,
    ) -> Result<Option<Vec<(String, i32)>>, Error> {
        let mut path = self.v2_path();
        path.push_str("/plan");
        path.push_str("/at");
        path.push_str(&format!("/{}", date));
        let resp = gloo_net::http::Request::get(&path).send().await?;
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

    //pub async fn fetch_plan(&self) -> Result<Option<Vec<(String, i32)>>, Error> {
    //    let mut path = self.v2_path();
    //    path.push_str("/plan");
    //    let resp = gloo_net::http::Request::get(&path).send().await?;
    //    if resp.status() != 200 {
    //        Err(format!("Status: {}", resp.status()).into())
    //    } else {
    //        debug!("We got a valid response back");
    //        let plan = resp
    //            .json::<PlanDataResponse>()
    //            .await
    //            .map_err(|e| format!("{}", e))?
    //            .as_success();
    //        Ok(plan)
    //    }
    //}

    pub async fn fetch_inventory_for_date(
        &self,
        date: &NaiveDate,
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
        path.push_str("/at");
        path.push_str(&format!("/{}", date));
        let resp = gloo_net::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
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

    pub async fn fetch_inventory_data(
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
        let resp = gloo_net::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
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
    pub async fn store_inventory_data_for_date(
        &self,
        filtered_ingredients: BTreeSet<IngredientKey>,
        modified_amts: BTreeMap<IngredientKey, String>,
        extra_items: Vec<(String, String)>,
        date: &NaiveDate,
    ) -> Result<(), Error> {
        let mut path = self.v2_path();
        path.push_str("/inventory");
        path.push_str("/at");
        path.push_str(&format!("/{}", date));
        let filtered_ingredients: Vec<IngredientKey> = filtered_ingredients.into_iter().collect();
        let modified_amts: Vec<(IngredientKey, String)> = modified_amts.into_iter().collect();
        debug!("Storing inventory data via API");
        let resp = gloo_net::http::Request::post(&path)
            .json(&(filtered_ingredients, modified_amts, extra_items))
            .expect("Failed to set body")
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

    #[instrument]
    pub async fn store_inventory_data(
        &self,
        filtered_ingredients: BTreeSet<IngredientKey>,
        modified_amts: BTreeMap<IngredientKey, String>,
        extra_items: Vec<(String, String)>,
    ) -> Result<(), Error> {
        let mut path = self.v2_path();
        path.push_str("/inventory");
        let filtered_ingredients: Vec<IngredientKey> = filtered_ingredients.into_iter().collect();
        let modified_amts: Vec<(IngredientKey, String)> = modified_amts.into_iter().collect();
        debug!("Storing inventory data via API");
        let resp = gloo_net::http::Request::post(&path)
            .json(&(filtered_ingredients, modified_amts, extra_items))
            .expect("Failed to set body")
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

    pub async fn fetch_staples(&self) -> Result<Option<String>, Error> {
        let mut path = self.v2_path();
        path.push_str("/staples");
        let resp = gloo_net::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            debug!("Invalid response back");
            Err(format!("Status: {}", resp.status()).into())
        } else {
            Ok(resp
                .json::<Response<Option<String>>>()
                .await
                .expect("Failed to parse staples json")
                .as_success()
                .unwrap())
        }
    }

    pub async fn store_staples<S: AsRef<str> + serde::Serialize>(
        &self,
        content: S,
    ) -> Result<(), Error> {
        let mut path = self.v2_path();
        path.push_str("/staples");
        let resp = gloo_net::http::Request::post(&path)
            .json(&content)
            .expect("Failed to set body")
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
