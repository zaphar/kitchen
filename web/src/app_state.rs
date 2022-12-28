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
use recipes::{parse, Ingredient, IngredientAccumulator, IngredientKey, Recipe, RecipeEntry};
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
    InitRecipeCounts(BTreeMap<String, usize>),
    UpdateRecipeCount(String, usize),
    InitExtras(BTreeSet<(String, String)>),
    AddExtra(String, String),
    RemoveExtra(String, String),
    InitRecipes(BTreeMap<String, Recipe>),
    SaveRecipe(RecipeEntry),
    SetRecipe(String, Recipe),
    RemoveRecipe(String),
    SetStaples(Option<Recipe>),
    SetCategoryMap(String),
    UpdateCategories,
    InitFilteredIngredient(BTreeSet<IngredientKey>),
    AddFilteredIngredient(IngredientKey),
    RemoveFilteredIngredient(IngredientKey),
    InitAmts(BTreeMap<IngredientKey, String>),
    UpdateAmt(IngredientKey, String),
    SetUserData(UserData),
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
            Message::InitRecipeCounts(map) => {
                original_copy.recipe_counts = map;
            }
            Message::UpdateRecipeCount(id, count) => {
                original_copy.recipe_counts.insert(id, count);
            }
            Message::InitExtras(set) => {
                original_copy.extras = set;
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
            Message::InitRecipes(recipes) => {
                original_copy.recipes = recipes;
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
            Message::UpdateCategories => {
                let store = self.0.clone();
                let mut original_copy = original_copy.clone();
                spawn_local_scoped(cx, async move {
                    if let Some(categories) = match store.get_categories().await {
                        Ok(js) => js,
                        Err(e) => {
                            error!(err=?e, "Failed to get categories.");
                            return;
                        }
                    } {
                        original_copy.category_map = categories;
                    };
                });
            }
            Message::InitFilteredIngredient(set) => {
                original_copy.filtered_ingredients = set;
            }
            Message::AddFilteredIngredient(key) => {
                original_copy.filtered_ingredients.insert(key);
            }
            Message::RemoveFilteredIngredient(key) => {
                original_copy.filtered_ingredients.remove(&key);
            }
            Message::InitAmts(map) => {
                original_copy.modified_amts = map;
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

#[derive(Debug)]
pub struct State {
    pub recipe_counts: RcSignal<BTreeMap<String, RcSignal<usize>>>,
    pub extras: RcSignal<Vec<(usize, (RcSignal<String>, RcSignal<String>))>>,
    pub staples: RcSignal<Option<Recipe>>,
    pub recipes: RcSignal<BTreeMap<String, Recipe>>,
    pub category_map: RcSignal<BTreeMap<String, String>>,
    pub filtered_ingredients: RcSignal<BTreeSet<IngredientKey>>,
    pub modified_amts: RcSignal<BTreeMap<IngredientKey, RcSignal<String>>>,
    pub auth: RcSignal<Option<UserData>>,
}

impl State {
    pub fn get_from_context(cx: Scope) -> std::rc::Rc<Self> {
        use_context::<std::rc::Rc<Self>>(cx).clone()
    }

    pub fn get_menu_list(&self) -> Vec<(String, RcSignal<usize>)> {
        self.recipe_counts
            .get()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .filter(|(_, v)| *(v.get_untracked()) != 0)
            .collect()
    }

    #[instrument(skip(self))]
    pub fn get_shopping_list(
        &self,
        show_staples: bool,
    ) -> BTreeMap<String, Vec<(Ingredient, BTreeSet<String>)>> {
        let mut acc = IngredientAccumulator::new();
        let recipe_counts = self.get_menu_list();
        for (idx, count) in recipe_counts.iter() {
            for _ in 0..*count.get_untracked() {
                acc.accumulate_from(
                    self.recipes
                        .get()
                        .get(idx)
                        .expect(&format!("No such recipe id exists: {}", idx)),
                );
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

    /// Retrieves the count for a recipe without triggering subscribers to the entire
    /// recipe count set.
    pub fn get_recipe_count_by_index(&self, key: &String) -> Option<RcSignal<usize>> {
        self.recipe_counts.get_untracked().get(key).cloned()
    }

    pub fn reset_recipe_counts(&self) {
        for (_, count) in self.recipe_counts.get_untracked().iter() {
            count.set(0);
        }
    }

    /// Set the recipe_count by index. Does not trigger subscribers to the entire set of recipe_counts.
    /// This does trigger subscribers of the specific recipe you are updating though.
    pub fn set_recipe_count_by_index(&self, key: &String, count: usize) -> RcSignal<usize> {
        let mut counts = self.recipe_counts.get_untracked().as_ref().clone();
        counts
            .entry(key.clone())
            .and_modify(|e| e.set(count))
            .or_insert_with(|| create_rc_signal(count));
        self.recipe_counts.set(counts);
        self.recipe_counts.get_untracked().get(key).unwrap().clone()
    }

    pub fn get_current_modified_amts(&self) -> BTreeMap<IngredientKey, String> {
        let mut modified_amts = BTreeMap::new();
        for (key, amt) in self.modified_amts.get_untracked().iter() {
            modified_amts.insert(key.clone(), amt.get_untracked().as_ref().clone());
        }
        modified_amts
    }

    pub fn reset_modified_amts(&self, modified_amts: BTreeMap<IngredientKey, String>) {
        let mut modified_amts_copy = self.modified_amts.get().as_ref().clone();
        for (key, amt) in modified_amts {
            modified_amts_copy
                .entry(key)
                .and_modify(|amt_signal| amt_signal.set(amt.clone()))
                .or_insert_with(|| create_rc_signal(amt));
        }
        self.modified_amts.set(modified_amts_copy);
    }
}
