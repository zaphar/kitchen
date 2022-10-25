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

use reqwasm;
use serde_json::{from_str, to_string};
use sycamore::prelude::*;
use tracing::{debug, error, info, instrument, warn};

use recipes::{parse, Recipe, RecipeEntry};

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
    if let Some(rs) = recipes {
        for r in rs {
            if !state.recipe_counts.get().contains_key(r.recipe_id()) {
                state.set_recipe_count_by_index(&r.recipe_id().to_owned(), 0);
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

    pub fn provide_context<S: Into<String>>(cx: Scope, root: S) {
        provide_context(cx, std::rc::Rc::new(Self::new(root.into())));
    }

    pub fn get_from_context(cx: Scope) -> std::rc::Rc<Self> {
        use_context::<std::rc::Rc<Self>>(cx).clone()
    }

    #[instrument]
    pub async fn get_categories(&self) -> Result<Option<String>, Error> {
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
            let resp = resp.json().await;
            Ok(Some(resp.map_err(|e| format!("{}", e))?))
        }
    }

    #[instrument]
    pub async fn get_recipes(&self) -> Result<Option<Vec<RecipeEntry>>, Error> {
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

    pub async fn get_recipe_text<S: AsRef<str>>(
        &self,
        id: S,
    ) -> Result<Option<RecipeEntry>, Error> {
        let mut path = self.root.clone();
        path.push_str("/recipe/");
        path.push_str(id.as_ref());
        let resp = reqwasm::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            Ok(resp.json().await.map_err(|e| format!("{}", e))?)
        }
    }

    #[instrument(skip(recipes), fields(count=recipes.len()))]
    pub async fn save_recipes(&self, recipes: Vec<RecipeEntry>) -> Result<(), Error> {
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
    pub async fn save_categories(&self, categories: String) -> Result<(), Error> {
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
