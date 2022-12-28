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

use sycamore::{futures::spawn_local, prelude::*};
use tracing::{debug, error, instrument, warn};

use client_api::UserData;
use recipes::{Ingredient, IngredientAccumulator, IngredientKey, Recipe};

use sycamore_state::{Handler, MessageMapper};

use crate::api::HttpStore;

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub recipe_counts: BTreeMap<String, usize>,
    pub extras: BTreeSet<(String, String)>,
    pub staples: Option<Recipe>,
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
            extras: BTreeSet::new(),
            staples: None,
            recipes: BTreeMap::new(),
            category_map: BTreeMap::new(),
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
    SetRecipe(String, Recipe),
    RemoveRecipe(String),
    SetStaples(Option<Recipe>),
    SetCategoryMap(BTreeMap<String, String>),
    InitFilteredIngredient(BTreeSet<IngredientKey>),
    AddFilteredIngredient(IngredientKey),
    RemoveFilteredIngredient(IngredientKey),
    InitAmts(BTreeMap<IngredientKey, String>),
    UpdateAmt(IngredientKey, String),
    SetUserData(UserData),
    UnsetUserData,
    SaveState,
}

pub struct StateMachine(HttpStore);

impl MessageMapper<Message, AppState> for StateMachine {
    #[instrument(skip_all, fields(?msg))]
    fn map(&self, msg: Message, original: &ReadSignal<AppState>) -> AppState {
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
            Message::RemoveRecipe(id) => {
                original_copy.recipes.remove(&id);
            }
            Message::SetCategoryMap(map) => {
                original_copy.category_map = map;
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
                spawn_local(async move {
                    if let Err(e) = store.save_app_state(original_copy).await {
                        error!(err=?e, "Error saving app state")
                    };
                });
            }
        }
        original_copy
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
