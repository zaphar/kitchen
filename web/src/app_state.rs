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

use client_api::UserData;
use recipes::{parse, IngredientKey, Recipe, RecipeEntry};
use serde_json::from_str;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use sycamore_state::{Handler, MessageMapper};
use tracing::{debug, error, info, instrument, warn};

use crate::api::HttpStore;
use crate::js_lib;

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub recipe_counts: BTreeMap<String, usize>,
    pub extras: BTreeSet<(String, String)>,
    pub staples: Option<Recipe>,
    pub recipes: BTreeMap<String, Recipe>,
    pub category_map: String,
    pub filtered_ingredients: BTreeSet<IngredientKey>,
    pub modified_amts: BTreeMap<IngredientKey, String>,
    pub auth: Option<UserData>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            recipe_counts: BTreeMap::new(),
            extras: BTreeSet::new(),
            staples: None,
            recipes: BTreeMap::new(),
            category_map: String::new(),
            filtered_ingredients: BTreeSet::new(),
            modified_amts: BTreeMap::new(),
            auth: None,
        }
    }
}

#[derive(Debug)]
pub enum Message {
    ResetRecipeCounts,
    UpdateRecipeCount(String, usize),
    AddExtra(String, String),
    RemoveExtra(String, String),
    SaveRecipe(RecipeEntry),
    SetRecipe(String, Recipe),
    // TODO(jwall): Remove this annotation when safe to do so.
    #[allow(dead_code)]
    RemoveRecipe(String),
    // TODO(jwall): Remove this annotation when safe to do so.
    #[allow(dead_code)]
    SetStaples(Option<Recipe>),
    SetCategoryMap(String),
    ResetInventory,
    AddFilteredIngredient(IngredientKey),
    UpdateAmt(IngredientKey, String),
    SetUserData(UserData),
    // TODO(jwall): Remove this annotation when safe to do so.
    #[allow(dead_code)]
    UnsetUserData,
    SaveState,
    LoadState,
}

pub struct StateMachine(HttpStore);

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

impl StateMachine {
    async fn load_state(store: HttpStore, original: &Signal<AppState>) {
        let mut state = original.get().as_ref().clone();
        info!("Synchronizing Recipes");
        // TODO(jwall): Make our caching logic using storage more robust.
        let recipe_entries = match store.get_recipes().await {
            Ok(recipe_entries) => {
                if let Ok((staples, recipes)) = filter_recipes(&recipe_entries) {
                    state.staples = staples;
                    if let Some(recipes) = recipes {
                        state.recipes = recipes;
                    }
                }
                recipe_entries
            }
            Err(err) => {
                error!(?err);
                None
            }
        };

        if let Ok(Some(plan)) = store.get_plan().await {
            // set the counts.
            let mut plan_map = BTreeMap::new();
            for (id, count) in plan {
                plan_map.insert(id, count as usize);
            }
            state.recipe_counts = plan_map;
        } else {
            // Initialize things to zero
            if let Some(rs) = recipe_entries {
                for r in rs {
                    state.recipe_counts.insert(r.recipe_id().to_owned(), 0);
                }
            }
        }
        info!("Checking for user_data in local storage");
        let storage = js_lib::get_storage();
        let user_data = storage
            .get("user_data")
            .expect("Couldn't read from storage");
        if let Some(data) = user_data {
            if let Ok(user_data) = from_str(&data) {
                state.auth = Some(user_data);
            }
        }
        info!("Synchronizing categories");
        match store.get_categories().await {
            Ok(Some(categories_content)) => {
                debug!(categories=?categories_content);
                state.category_map = categories_content;
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
            Ok((filtered_ingredients, modified_amts, extra_items)) => {
                state.modified_amts = modified_amts;
                state.filtered_ingredients = filtered_ingredients;
                state.extras = BTreeSet::from_iter(extra_items);
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
        original.set(state);
    }
}

impl MessageMapper<Message, AppState> for StateMachine {
    #[instrument(skip_all, fields(?msg))]
    fn map<'ctx>(&self, cx: Scope<'ctx>, msg: Message, original: &'ctx Signal<AppState>) {
        let mut original_copy = original.get().as_ref().clone();
        match msg {
            Message::ResetRecipeCounts => {
                let mut map = BTreeMap::new();
                for (id, _) in original_copy.recipes.iter() {
                    map.insert(id.clone(), 0);
                }
                original_copy.recipe_counts = map;
            }
            Message::UpdateRecipeCount(id, count) => {
                original_copy.recipe_counts.insert(id, count);
            }
            Message::AddExtra(amt, name) => {
                original_copy.extras.insert((amt, name));
            }
            Message::RemoveExtra(amt, name) => {
                original_copy.extras.remove(&(amt, name));
            }
            Message::SetStaples(staples) => {
                original_copy.staples = staples;
            }
            Message::SetRecipe(id, recipe) => {
                original_copy.recipes.insert(id, recipe);
            }
            Message::SaveRecipe(entry) => {
                let recipe =
                    parse::as_recipe(entry.recipe_text()).expect("Failed to parse RecipeEntry");
                original_copy
                    .recipes
                    .insert(entry.recipe_id().to_owned(), recipe);
                let store = self.0.clone();
                original_copy
                    .recipe_counts
                    .insert(entry.recipe_id().to_owned(), 0);
                spawn_local_scoped(cx, async move {
                    if let Err(e) = store.save_recipes(vec![entry]).await {
                        error!(err=?e, "Unable to save Recipe");
                    }
                });
            }
            Message::RemoveRecipe(id) => {
                original_copy.recipes.remove(&id);
            }
            Message::SetCategoryMap(category_text) => {
                let store = self.0.clone();
                original_copy.category_map = category_text.clone();
                spawn_local_scoped(cx, async move {
                    if let Err(e) = store.save_categories(category_text).await {
                        error!(?e, "Failed to save categories");
                    }
                });
            }
            Message::ResetInventory => {
                original_copy.filtered_ingredients = BTreeSet::new();
                original_copy.modified_amts = BTreeMap::new();
                original_copy.extras = BTreeSet::new();
            }
            Message::AddFilteredIngredient(key) => {
                original_copy.filtered_ingredients.insert(key);
            }
            Message::UpdateAmt(key, amt) => {
                original_copy.modified_amts.insert(key, amt);
            }
            Message::SetUserData(user_data) => {
                original_copy.auth = Some(user_data);
            }
            Message::UnsetUserData => {
                original_copy.auth = None;
            }
            Message::SaveState => {
                let store = self.0.clone();
                let original_copy = original_copy.clone();
                spawn_local_scoped(cx, async move {
                    if let Err(e) = store.save_app_state(original_copy).await {
                        error!(err=?e, "Error saving app state")
                    };
                });
            }
            Message::LoadState => {
                let store = self.0.clone();
                spawn_local_scoped(cx, async move {
                    Self::load_state(store, original).await;
                });
                return;
            }
        }
        original.set(original_copy);
    }
}

pub type StateHandler<'ctx> = &'ctx Handler<'ctx, StateMachine, AppState, Message>;

pub fn get_state_handler<'ctx>(
    cx: Scope<'ctx>,
    initial: AppState,
    store: HttpStore,
) -> StateHandler<'ctx> {
    Handler::new(cx, initial, StateMachine(store))
}
