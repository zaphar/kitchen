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

use recipes::{Ingredient, IngredientAccumulator, Recipe};

pub struct State {
    pub recipe_counts: RcSignal<BTreeMap<String, usize>>,
    pub extras: RcSignal<Vec<(usize, (RcSignal<String>, RcSignal<String>))>>,
    pub staples: RcSignal<Option<Recipe>>,
    pub recipes: RcSignal<BTreeMap<String, Recipe>>,
    pub category_map: RcSignal<BTreeMap<String, String>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            recipe_counts: create_rc_signal(BTreeMap::new()),
            extras: create_rc_signal(Vec::new()),
            staples: create_rc_signal(None),
            recipes: create_rc_signal(BTreeMap::new()),
            category_map: create_rc_signal(BTreeMap::new()),
        }
    }

    pub fn provide_context(cx: Scope) {
        provide_context(cx, std::rc::Rc::new(Self::new()));
    }

    pub fn get_from_context(cx: Scope) -> std::rc::Rc<Self> {
        use_context::<std::rc::Rc<Self>>(cx).clone()
    }

    pub fn get_menu_list(&self) -> Vec<(String, usize)> {
        self.recipe_counts
            .get()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .filter(|(_, v)| *v != 0)
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
            for _ in 0..*count {
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

    pub fn get_recipe_count_by_index(&self, key: &String) -> Option<usize> {
        self.recipe_counts.get().get(key).cloned()
    }

    pub fn set_recipe_count_by_index(&self, key: &String, count: usize) -> usize {
        let mut counts = self.recipe_counts.get().as_ref().clone();
        counts.insert(key.clone(), count);
        self.recipe_counts.set(counts);
        count
    }
}
