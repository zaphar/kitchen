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

use sycamore::prelude::*;
use tracing::{debug, instrument, warn};

use recipes::{Ingredient, IngredientAccumulator, IngredientKey, Recipe};

#[derive(Debug)]
pub struct State {
    pub recipe_counts: RcSignal<BTreeMap<String, RcSignal<usize>>>,
    pub extras: RcSignal<Vec<(usize, (RcSignal<String>, RcSignal<String>))>>,
    pub staples: RcSignal<Option<Recipe>>,
    pub recipes: RcSignal<BTreeMap<String, Recipe>>,
    pub category_map: RcSignal<BTreeMap<String, String>>,
    pub filtered_ingredients: RcSignal<BTreeSet<IngredientKey>>,
    pub modified_amts: RcSignal<BTreeMap<IngredientKey, RcSignal<String>>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            recipe_counts: create_rc_signal(BTreeMap::new()),
            extras: create_rc_signal(Vec::new()),
            staples: create_rc_signal(None),
            recipes: create_rc_signal(BTreeMap::new()),
            category_map: create_rc_signal(BTreeMap::new()),
            filtered_ingredients: create_rc_signal(BTreeSet::new()),
            modified_amts: create_rc_signal(BTreeMap::new()),
        }
    }

    pub fn provide_context(cx: Scope) {
        provide_context(cx, std::rc::Rc::new(Self::new()));
    }

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
