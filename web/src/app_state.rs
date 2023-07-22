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

use chrono::NaiveDate;
use client_api::UserData;
use recipes::{parse, Ingredient, IngredientKey, Recipe, RecipeEntry};
use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use sycamore_state::{Handler, MessageMapper};
use tracing::{debug, error, info, instrument, warn};
use wasm_bindgen::throw_str;

use crate::{
    api::{HttpStore, LocalStore},
    components, linear::LinearSignal,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    pub recipe_counts: BTreeMap<String, usize>,
    pub recipe_categories: BTreeMap<String, String>,
    pub extras: Vec<(String, String)>,
    #[serde(skip)] // FIXME(jwall): This should really be storable I think?
    pub staples: Option<BTreeSet<Ingredient>>,
    #[serde(skip)] // FIXME(jwall): This should really be storable I think?
    pub recipes: BTreeMap<String, Recipe>,
    pub category_map: BTreeMap<String, String>,
    pub filtered_ingredients: BTreeSet<IngredientKey>,
    pub modified_amts: BTreeMap<IngredientKey, String>,
    pub auth: Option<UserData>,
    pub plan_dates: BTreeSet<NaiveDate>,
    pub selected_plan_date: Option<NaiveDate>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            recipe_counts: BTreeMap::new(),
            recipe_categories: BTreeMap::new(),
            extras: Vec::new(),
            staples: None,
            recipes: BTreeMap::new(),
            category_map: BTreeMap::new(),
            filtered_ingredients: BTreeSet::new(),
            modified_amts: BTreeMap::new(),
            auth: None,
            plan_dates: BTreeSet::new(),
            selected_plan_date: None,
        }
    }
}

pub enum Message {
    ResetRecipeCounts,
    UpdateRecipeCount(String, usize),
    AddExtra(String, String),
    RemoveExtra(usize),
    UpdateExtra(usize, String, String),
    SaveRecipe(RecipeEntry, Option<Box<dyn FnOnce()>>),
    RemoveRecipe(String, Option<Box<dyn FnOnce()>>),
    UpdateCategory(String, String, Option<Box<dyn FnOnce()>>),
    ResetInventory,
    AddFilteredIngredient(IngredientKey),
    UpdateAmt(IngredientKey, String),
    SetUserData(UserData),
    SaveState(Option<Box<dyn FnOnce()>>),
    LoadState(Option<Box<dyn FnOnce()>>),
    UpdateStaples(String, Option<Box<dyn FnOnce()>>),
    DeletePlan(NaiveDate, Option<Box<dyn FnOnce()>>),
    SelectPlanDate(NaiveDate, Option<Box<dyn FnOnce()>>),
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
            Self::SaveRecipe(arg0, _) => f.debug_tuple("SaveRecipe").field(arg0).finish(),
            Self::RemoveRecipe(arg0, _) => f.debug_tuple("SetCategoryMap").field(arg0).finish(),
            Self::UpdateCategory(i, c, _) => {
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
            Self::UpdateStaples(arg, _) => f.debug_tuple("UpdateStaples").field(arg).finish(),
            Self::SelectPlanDate(arg, _) => f.debug_tuple("SelectPlanDate").field(arg).finish(),
            Self::DeletePlan(arg, _) => f.debug_tuple("DeletePlan").field(arg).finish(),
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
        // TODO(jwall): Load plan state from local_store first.
        let original: LinearSignal<AppState> = original.into();
        let mut state = original.get().as_ref().clone();
        info!("Synchronizing Recipes");
        let recipe_entries = &store.fetch_recipes().await?;
        let recipes = parse_recipes(&recipe_entries)?;
        debug!(?recipes, "Parsed Recipes");
        if let Some(recipes) = recipes {
            state.recipes = recipes;
        };

        info!("Synchronizing staples");
        state.staples = if let Some(content) = store.fetch_staples().await? {
            // now we need to parse staples as ingredients
            let mut staples = parse::as_ingredient_list(&content)?;
            Some(staples.drain(0..).collect())
        } else {
            Some(BTreeSet::new())
        };

        info!("Synchronizing recipe");
        if let Some(recipe_entries) = recipe_entries {
            local_store.set_all_recipes(recipe_entries);
            state.recipe_categories = recipe_entries
                .iter()
                .map(|entry| {
                    debug!(recipe_entry=?entry, "Getting recipe category");
                    (
                        entry.recipe_id().to_owned(),
                        entry
                            .category()
                            .cloned()
                            .unwrap_or_else(|| "Entree".to_owned()),
                    )
                })
                .collect::<BTreeMap<String, String>>();
        }

        info!("Fetching meal plan list");
        if let Some(mut plan_dates) = store.fetch_plan_dates().await? {
            debug!(?plan_dates, "meal plan list");
            state.plan_dates = BTreeSet::from_iter(plan_dates.drain(0..));
        }

        info!("Synchronizing meal plan");
        let plan = if let Some(ref cached_plan_date) = state.selected_plan_date {
            store
                .fetch_plan_for_date(cached_plan_date)
                .await?
                .or_else(|| Some(Vec::new()))
        } else {
            None
        };
        if let Some(plan) = plan {
            // set the counts.
            let mut plan_map = BTreeMap::new();
            for (id, count) in plan {
                plan_map.insert(id, count as usize);
            }
            state.recipe_counts = plan_map;
            for (id, _) in state.recipes.iter() {
                if !state.recipe_counts.contains_key(id) {
                    state.recipe_counts.insert(id.clone(), 0);
                }
            }
        } else {
            // Initialize things to zero.
            if let Some(rs) = recipe_entries {
                for r in rs {
                    state.recipe_counts.insert(r.recipe_id().to_owned(), 0);
                }
            }
        }
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
                let category_map = BTreeMap::from_iter(categories_content.drain(0..));
                state.category_map = category_map;
            }
            Ok(None) => {
                warn!("There is no category file");
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
        let inventory_data = if let Some(cached_plan_date) = &state.selected_plan_date {
            store.fetch_inventory_for_date(cached_plan_date).await
        } else {
            store.fetch_inventory_data().await
        };
        info!("Synchronizing inventory data");
        match inventory_data {
            Ok((filtered_ingredients, modified_amts, extra_items)) => {
                state.modified_amts = modified_amts;
                state.filtered_ingredients = filtered_ingredients;
                state.extras = extra_items;
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
        // Finally we store all of this app state back to our localstore
        local_store.store_app_state(&state);
        original.update(state);
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
                original_copy.recipe_counts = map;
            }
            Message::UpdateRecipeCount(id, count) => {
                original_copy.recipe_counts.insert(id, count);
            }
            Message::AddExtra(amt, name) => {
                original_copy.extras.push((amt, name));
            }
            Message::RemoveExtra(idx) => {
                original_copy.extras.remove(idx);
            }
            Message::UpdateExtra(idx, amt, name) => match original_copy.extras.get_mut(idx) {
                Some(extra) => {
                    extra.0 = amt;
                    extra.1 = name;
                }
                None => {
                    throw_str("Attempted to remove extra that didn't exist");
                }
            },
            Message::SaveRecipe(entry, callback) => {
                let recipe =
                    parse::as_recipe(entry.recipe_text()).expect("Failed to parse RecipeEntry");
                original_copy
                    .recipes
                    .insert(entry.recipe_id().to_owned(), recipe);
                if !original_copy.recipe_counts.contains_key(entry.recipe_id()) {
                    original_copy
                        .recipe_counts
                        .insert(entry.recipe_id().to_owned(), 0);
                }
                if let Some(cat) = entry.category().cloned() {
                    original_copy
                        .recipe_categories
                        .entry(entry.recipe_id().to_owned())
                        .and_modify(|c| *c = cat.clone())
                        .or_insert(cat);
                }
                let store = self.store.clone();
                self.local_store.set_recipe_entry(&entry);
                spawn_local_scoped(cx, async move {
                    if let Err(e) = store.store_recipes(vec![entry]).await {
                        // FIXME(jwall): We should have a global way to trigger error messages
                        error!(err=?e, "Unable to save Recipe");
                        // FIXME(jwall): This should be an error message
                        components::toast::error_message(cx, "Failed to save Recipe", None);
                    } else {
                        components::toast::message(cx, "Saved Recipe", None);
                    }
                    callback.map(|f| f());
                });
            }
            Message::RemoveRecipe(recipe, callback) => {
                original_copy.recipe_counts.remove(&recipe);
                original_copy.recipes.remove(&recipe);
                self.local_store.delete_recipe_entry(&recipe);
                let store = self.store.clone();
                spawn_local_scoped(cx, async move {
                    if let Err(err) = store.delete_recipe(&recipe).await {
                        error!(?err, "Failed to delete recipe");
                        components::toast::error_message(cx, "Unable to delete recipe", None);
                    } else {
                        components::toast::message(cx, "Deleted Recipe", None);
                    }
                    callback.map(|f| f());
                });
            }
            Message::UpdateCategory(ingredient, category, callback) => {
                original_copy
                    .category_map
                    .insert(ingredient.clone(), category.clone());
                let store = self.store.clone();
                spawn_local_scoped(cx, async move {
                    if let Err(e) = store.store_categories(&vec![(ingredient, category)]).await {
                        error!(?e, "Failed to save categories");
                    }
                    callback.map(|f| f());
                });
            }
            Message::ResetInventory => {
                original_copy.filtered_ingredients = BTreeSet::new();
                original_copy.modified_amts = BTreeMap::new();
                original_copy.extras = Vec::new();
                components::toast::message(cx, "Reset Inventory", None);
            }
            Message::AddFilteredIngredient(key) => {
                original_copy.filtered_ingredients.insert(key);
            }
            Message::UpdateAmt(key, amt) => {
                original_copy.modified_amts.insert(key, amt);
            }
            Message::SetUserData(user_data) => {
                self.local_store.set_user_data(Some(&user_data));
                original_copy.auth = Some(user_data);
            }
            Message::SaveState(f) => {
                let mut original_copy = original_copy.clone();
                let store = self.store.clone();
                let local_store = self.local_store.clone();
                spawn_local_scoped(cx, async move {
                    if original_copy.selected_plan_date.is_none() {
                        original_copy.selected_plan_date = Some(chrono::Local::now().date_naive());
                    }
                    original_copy.plan_dates.insert(
                        original_copy
                            .selected_plan_date
                            .as_ref()
                            .map(|d| d.clone())
                            .unwrap(),
                    );
                    if let Err(e) = store.store_app_state(&original_copy).await {
                        error!(err=?e, "Error saving app state");
                        components::toast::error_message(cx, "Failed to save user state", None);
                    } else {
                        components::toast::message(cx, "Saved user state", None);
                    };
                    local_store.store_app_state(&original_copy);
                    original.set(original_copy);
                    f.map(|f| f());
                });
                // NOTE(jwall): We set the original signal in the async above
                // so we return immediately here.
                return;
            }
            Message::LoadState(f) => {
                let store = self.store.clone();
                let local_store = self.local_store.clone();
                debug!("Loading user state.");
                spawn_local_scoped(cx, async move {
                    if let Err(err) = Self::load_state(&store, &local_store, original.clone()).await
                    {
                        error!(?err, "Failed to load user state");
                        components::toast::error_message(cx, "Failed to load_state.", None);
                    } else {
                        components::toast::message(cx, "Loaded user state", None);
                    }
                    f.map(|f| f());
                });
                return;
            }
            Message::UpdateStaples(content, callback) => {
                let store = self.store.clone();
                spawn_local_scoped(cx, async move {
                    if let Err(err) = store.store_staples(content).await {
                        error!(?err, "Failed to store staples");
                        components::toast::error_message(cx, "Failed to store staples", None);
                    } else {
                        components::toast::message(cx, "Updated staples", None);
                        callback.map(|f| f());
                    }
                });
                return;
            }
            Message::SelectPlanDate(date, callback) => {
                let store = self.store.clone();
                let local_store = self.local_store.clone();
                spawn_local_scoped(cx, async move {
                    if let Some(mut plan) = store
                        .fetch_plan_for_date(&date)
                        .await
                        .expect("Failed to fetch plan for date")
                    {
                        // Note(jwall): This is a little unusual but because this
                        // is async code we can't rely on the set below.
                        original_copy.recipe_counts =
                            BTreeMap::from_iter(plan.drain(0..).map(|(k, v)| (k, v as usize)));
                    }
                    let (filtered, modified, extras) = store
                        .fetch_inventory_for_date(&date)
                        .await
                        .expect("Failed to fetch inventory_data for date");
                    original_copy.plan_dates.insert(date.clone());
                    original_copy.modified_amts = modified;
                    original_copy.filtered_ingredients = filtered;
                    original_copy.extras = extras;
                    original_copy.selected_plan_date = Some(date.clone());
                    store
                        .store_plan_for_date(vec![], &date)
                        .await
                        .expect("Failed to init meal plan for date");
                    local_store.store_app_state(&original_copy);
                    original.set(original_copy);

                    callback.map(|f| f());
                });
                // NOTE(jwall): Because we do our signal set above in the async block
                // we have to return here to avoid lifetime issues and double setting
                // the original signal.
                return;
            }
            Message::DeletePlan(date, callback) => {
                let store = self.store.clone();
                let local_store = self.local_store.clone();
                spawn_local_scoped(cx, async move {
                    if let Err(err) = store.delete_plan_for_date(&date).await {
                        components::toast::error_message(
                            cx,
                            "Failed to delete meal plan for date",
                            None,
                        );
                        error!(?err, "Error deleting plan");
                    } else {
                        original_copy.plan_dates.remove(&date);
                        // Reset all meal planning state;
                        let _ = original_copy.recipe_counts.iter_mut().map(|(_, v)| *v = 0);
                        original_copy.filtered_ingredients = BTreeSet::new();
                        original_copy.modified_amts = BTreeMap::new();
                        original_copy.extras = Vec::new();
                        local_store.store_app_state(&original_copy);
                        original.set(original_copy);
                        components::toast::message(cx, "Deleted Plan", None);

                        callback.map(|f| f());
                    }
                });
                // NOTE(jwall): Because we do our signal set above in the async block
                // we have to return here to avoid lifetime issues and double setting
                // the original signal.
                return;
            }
        }
        self.local_store.store_app_state(&original_copy);
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
