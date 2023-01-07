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
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use client_api::UserData;
use recipes::{parse, Ingredient, IngredientKey, Recipe, RecipeEntry};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use sycamore_state::{Handler, MessageMapper};
use tracing::{debug, error, info, instrument, warn};
use wasm_bindgen::throw_str;

use crate::api::{HttpStore, LocalStore};

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub recipe_counts: BTreeMap<String, usize>,
    pub extras: Vec<(String, String)>,
    pub staples: Option<BTreeSet<Ingredient>>,
    pub recipes: BTreeMap<String, Recipe>,
    pub category_map: BTreeMap<String, String>,
    pub filtered_ingredients: BTreeSet<IngredientKey>,
    pub modified_amts: BTreeMap<IngredientKey, String>,
    pub auth: Option<UserData>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            recipe_counts: BTreeMap::new(),
            extras: Vec::new(),
            staples: None,
            recipes: BTreeMap::new(),
            category_map: BTreeMap::new(),
            filtered_ingredients: BTreeSet::new(),
            modified_amts: BTreeMap::new(),
            auth: None,
        }
    }
}

pub enum Message {
    ResetRecipeCounts,
    UpdateRecipeCount(String, usize),
    AddExtra(String, String),
    RemoveExtra(usize),
    UpdateExtra(usize, String, String),
    SaveRecipe(RecipeEntry),
    SetRecipe(String, Recipe),
    RemoveRecipe(String),
    UpdateCategory(String, String),
    ResetInventory,
    AddFilteredIngredient(IngredientKey),
    UpdateAmt(IngredientKey, String),
    SetUserData(UserData),
    SaveState(Option<Box<dyn FnOnce()>>),
    LoadState(Option<Box<dyn FnOnce()>>),
    UpdateStaples(String),
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ResetRecipeCounts => write!(f, "ResetRecipeCounts"),
            Self::UpdateRecipeCount(arg0, arg1) => f
                .debug_tuple("UpdateRecipeCount")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::AddExtra(arg0, arg1) => {
                f.debug_tuple("AddExtra").field(arg0).field(arg1).finish()
            }
            Self::RemoveExtra(arg0) => f.debug_tuple("RemoveExtra").field(arg0).finish(),
            Self::UpdateExtra(arg0, arg1, arg2) => f
                .debug_tuple("UpdateExtra")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::SaveRecipe(arg0) => f.debug_tuple("SaveRecipe").field(arg0).finish(),
            Self::SetRecipe(arg0, arg1) => {
                f.debug_tuple("SetRecipe").field(arg0).field(arg1).finish()
            }
            Self::RemoveRecipe(arg0) => f.debug_tuple("SetCategoryMap").field(arg0).finish(),
            Self::UpdateCategory(i, c) => {
                f.debug_tuple("UpdateCategory").field(i).field(c).finish()
            }
            Self::ResetInventory => write!(f, "ResetInventory"),
            Self::AddFilteredIngredient(arg0) => {
                f.debug_tuple("AddFilteredIngredient").field(arg0).finish()
            }
            Self::UpdateAmt(arg0, arg1) => {
                f.debug_tuple("UpdateAmt").field(arg0).field(arg1).finish()
            }
            Self::SetUserData(arg0) => f.debug_tuple("SetUserData").field(arg0).finish(),
            Self::SaveState(_) => write!(f, "SaveState"),
            Self::LoadState(_) => write!(f, "LoadState"),
            Self::UpdateStaples(arg) => f.debug_tuple("UpdateStaples").field(arg).finish(),
        }
    }
}

pub struct StateMachine {
    store: HttpStore,
    local_store: LocalStore,
}

#[instrument]
fn parse_recipes(
    recipe_entries: &Option<Vec<RecipeEntry>>,
) -> Result<Option<BTreeMap<String, Recipe>>, String> {
    match recipe_entries {
        Some(parsed) => {
            let mut parsed_map = BTreeMap::new();
            for r in parsed {
                let recipe = match parse::as_recipe(&r.recipe_text()) {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Error parsing recipe {}", e);
                        continue;
                    }
                };
                parsed_map.insert(r.recipe_id().to_owned(), recipe);
            }
            Ok(Some(parsed_map))
        }
        None => Ok(None),
    }
}

impl StateMachine {
    pub fn new(store: HttpStore, local_store: LocalStore) -> Self {
        Self { store, local_store }
    }

    async fn load_state(
        store: &HttpStore,
        local_store: &LocalStore,
        original: &Signal<AppState>,
    ) -> Result<(), crate::api::Error> {
        let mut state = original.get().as_ref().clone();
        info!("Synchronizing Recipes");
        let recipe_entries = &store.fetch_recipes().await?;
        let recipes = parse_recipes(&recipe_entries)?;

        if let Some(recipes) = recipes {
            state.recipes = recipes;
        };

        state.staples = if let Some(content) = store.fetch_staples().await? {
            local_store.set_staples(&content);
            // now we need to parse staples as ingredients
            let mut staples = parse::as_ingredient_list(&content)?;
            Some(staples.drain(0..).collect())
        } else {
            if let Some(content) = local_store.get_staples() {
                let mut staples = parse::as_ingredient_list(&content)?;
                Some(staples.drain(0..).collect())
            } else {
                None
            }
        };

        if let Some(recipe_entries) = recipe_entries {
            local_store.set_all_recipes(recipe_entries);
        }

        let plan = store.fetch_plan().await?;
        if let Some(plan) = plan {
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
        let plan = state
            .recipe_counts
            .iter()
            .map(|(k, v)| (k.clone(), *v as i32))
            .collect::<Vec<(String, i32)>>();
        local_store.store_plan(&plan);
        info!("Checking for user account data");
        if let Some(user_data) = store.fetch_user_data().await {
            debug!("Successfully got account data from server");
            local_store.set_user_data(Some(&user_data));
            state.auth = Some(user_data);
        } else {
            debug!("Using account data from local store");
            let user_data = local_store.get_user_data();
            state.auth = user_data;
        }
        info!("Synchronizing categories");
        match store.fetch_categories().await {
            Ok(Some(mut categories_content)) => {
                debug!(categories=?categories_content);
                local_store.set_categories(Some(&categories_content));
                let category_map = BTreeMap::from_iter(categories_content.drain(0..));
                state.category_map = category_map;
            }
            Ok(None) => {
                warn!("There is no category file");
                local_store.set_categories(None);
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
        info!("Synchronizing inventory data");
        match store.fetch_inventory_data().await {
            Ok((filtered_ingredients, modified_amts, extra_items)) => {
                local_store.set_inventory_data((
                    &filtered_ingredients,
                    &modified_amts,
                    &extra_items,
                ));
                state.modified_amts = modified_amts;
                state.filtered_ingredients = filtered_ingredients;
                state.extras = extra_items;
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
        original.set(state);
        Ok(())
    }
}

impl MessageMapper<Message, AppState> for StateMachine {
    #[instrument(skip_all, fields(?msg))]
    fn map<'ctx>(&self, cx: Scope<'ctx>, msg: Message, original: &'ctx Signal<AppState>) {
        let mut original_copy = original.get().as_ref().clone();
        debug!("handling state message");
        match msg {
            Message::ResetRecipeCounts => {
                let mut map = BTreeMap::new();
                for (id, _) in original_copy.recipes.iter() {
                    map.insert(id.clone(), 0);
                }
                let plan: Vec<(String, i32)> =
                    map.iter().map(|(s, i)| (s.clone(), *i as i32)).collect();
                self.local_store.store_plan(&plan);
                original_copy.recipe_counts = map;
            }
            Message::UpdateRecipeCount(id, count) => {
                original_copy.recipe_counts.insert(id, count);
                let plan: Vec<(String, i32)> = original_copy
                    .recipe_counts
                    .iter()
                    .map(|(s, i)| (s.clone(), *i as i32))
                    .collect();
                self.local_store.store_plan(&plan);
            }
            Message::AddExtra(amt, name) => {
                original_copy.extras.push((amt, name));
                self.local_store.set_inventory_data((
                    &original_copy.filtered_ingredients,
                    &original_copy.modified_amts,
                    &original_copy.extras,
                ))
            }
            Message::RemoveExtra(idx) => {
                original_copy.extras.remove(idx);
                self.local_store.set_inventory_data((
                    &original_copy.filtered_ingredients,
                    &original_copy.modified_amts,
                    &original_copy.extras,
                ))
            }
            Message::UpdateExtra(idx, amt, name) => {
                match original_copy.extras.get_mut(idx) {
                    Some(extra) => {
                        extra.0 = amt;
                        extra.1 = name;
                    }
                    None => {
                        throw_str("Attempted to remove extra that didn't exist");
                    }
                }
                self.local_store.set_inventory_data((
                    &original_copy.filtered_ingredients,
                    &original_copy.modified_amts,
                    &original_copy.extras,
                ))
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
                original_copy
                    .recipe_counts
                    .insert(entry.recipe_id().to_owned(), 0);
                let store = self.store.clone();
                self.local_store.set_recipe_entry(&entry);
                spawn_local_scoped(cx, async move {
                    if let Err(e) = store.store_recipes(vec![entry]).await {
                        error!(err=?e, "Unable to save Recipe");
                    }
                });
            }
            Message::RemoveRecipe(recipe) => {
                original_copy.recipe_counts.remove(&recipe);
                original_copy.recipes.remove(&recipe);
                self.local_store.delete_recipe_entry(&recipe);
                let store = self.store.clone();
                spawn_local_scoped(cx, async move {
                    if let Err(err) = store.delete_recipe(&recipe).await {
                        error!(?err, "Failed to delete recipe");
                    }
                });
            }
            Message::UpdateCategory(ingredient, category) => {
                self.local_store
                    .set_categories(Some(&vec![(ingredient.clone(), category.clone())]));
                original_copy
                    .category_map
                    .insert(ingredient.clone(), category.clone());
                let store = self.store.clone();
                spawn_local_scoped(cx, async move {
                    if let Err(e) = store.store_categories(&vec![(ingredient, category)]).await {
                        error!(?e, "Failed to save categories");
                    }
                });
            }
            Message::ResetInventory => {
                original_copy.filtered_ingredients = BTreeSet::new();
                original_copy.modified_amts = BTreeMap::new();
                original_copy.extras = Vec::new();
                self.local_store.set_inventory_data((
                    &original_copy.filtered_ingredients,
                    &original_copy.modified_amts,
                    &original_copy.extras,
                ));
            }
            Message::AddFilteredIngredient(key) => {
                original_copy.filtered_ingredients.insert(key);
                self.local_store.set_inventory_data((
                    &original_copy.filtered_ingredients,
                    &original_copy.modified_amts,
                    &original_copy.extras,
                ));
            }
            Message::UpdateAmt(key, amt) => {
                original_copy.modified_amts.insert(key, amt);
                self.local_store.set_inventory_data((
                    &original_copy.filtered_ingredients,
                    &original_copy.modified_amts,
                    &original_copy.extras,
                ));
            }
            Message::SetUserData(user_data) => {
                self.local_store.set_user_data(Some(&user_data));
                original_copy.auth = Some(user_data);
            }
            Message::SaveState(f) => {
                let original_copy = original_copy.clone();
                let store = self.store.clone();
                spawn_local_scoped(cx, async move {
                    if let Err(e) = store.store_app_state(original_copy).await {
                        error!(err=?e, "Error saving app state")
                    };
                    f.map(|f| f());
                });
            }
            Message::LoadState(f) => {
                let store = self.store.clone();
                let local_store = self.local_store.clone();
                spawn_local_scoped(cx, async move {
                    Self::load_state(&store, &local_store, original)
                        .await
                        .expect("Failed to load_state.");
                    local_store.set_inventory_data((
                        &original.get().filtered_ingredients,
                        &original.get().modified_amts,
                        &original.get().extras,
                    ));
                    f.map(|f| f());
                });
                return;
            }
            Message::UpdateStaples(content) => {
                let store = self.store.clone();
                let local_store = self.local_store.clone();
                spawn_local_scoped(cx, async move {
                    local_store.set_staples(&content);
                    store
                        .store_staples(content)
                        .await
                        .expect("Failed to store staples");
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
    Handler::new(cx, initial, StateMachine::new(store, LocalStore::new()))
}
