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
use crate::service::AppService;
use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};

use recipes::{Ingredient, IngredientKey};
use sycamore::{context::use_context, prelude::*};

#[component(ShoppingList<G>)]
pub fn shopping_list() -> View<G> {
    let app_service = use_context::<AppService>();
    let filtered_keys = Signal::new(HashSet::new());
    let ingredients_map = Signal::new(BTreeMap::new());
    let modified_amts = Signal::new(HashMap::new());
    create_effect(cloned!((app_service, ingredients_map) => move || {
        ingredients_map.set(app_service.get_shopping_list());
    }));
    let ingredients = create_memo(cloned!((ingredients_map, filtered_keys) => move || {
        ingredients_map
            .get()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .filter(|(k, _v)| !filtered_keys.get().contains(k))
            .collect::<Vec<(IngredientKey, Ingredient)>>()
    }));
    let table_view = Signal::new(View::empty());
    create_effect(
        cloned!((table_view, ingredients, filtered_keys, modified_amts) => move || {
            if ingredients.get().len() > 0 {
                let t = view ! {
                    table(class="pad-top shopping-list page-breaker table table-striped table-condensed table-responsive") {
                        tr {
                            th { " Quantity " }
                            th { " Ingredient " }
                        }
                        tbody {Indexed(IndexedProps{
                            iterable: ingredients.clone(),
                            template: cloned!((filtered_keys, modified_amts) => move |(k, i)| {
                                let mut modified_amt_set = (*modified_amts.get()).clone();
                                let amt = modified_amt_set.entry(k.clone()).or_insert(Signal::new(format!("{}", i.amt.normalize()))).clone();
                                modified_amts.set(modified_amt_set);
                                let name = i.name;
                                let form = i.form.map(|form| format!("({})", form)).unwrap_or_default();
                                view! {
                                    tr {
                                        td { input(bind:value=amt.clone(), class="ingredient-count-sel", type="text") }
                                        td {input(type="button", class="no-print", value="X", on:click=cloned!((filtered_keys) => move |_| {
                                            let mut keyset = (*filtered_keys.get()).clone();
                                            keyset.insert(k.clone());
                                            filtered_keys.set(keyset);
                                        }))  " " (name) " " (form) }
                                    }
                                }
                            }),
                        })}
                    }
                };
                table_view.set(t);
            } else {
                table_view.set(View::empty());
            }
        }),
    );
    // TODO(jwall): Sort by categories and names.
    view! {
        h1 { "Shopping List " }
        input(type="button", value="Reset", class="no-print", on:click=cloned!((ingredients_map, filtered_keys, app_service, modified_amts) => move |_| {
            ingredients_map.set(app_service.get_shopping_list());
            // clear the filter_signal
            filtered_keys.set(HashSet::new());
            modified_amts.set(HashMap::new());
        }))
        (table_view.get().as_ref().clone())
    }
}
