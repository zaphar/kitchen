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
use std::collections::{BTreeMap, BTreeSet};

use sycamore::{context::use_context, prelude::*};
use tracing::{debug, instrument};

#[instrument]
#[component(ShoppingList<G>)]
pub fn shopping_list() -> View<G> {
    let app_service = use_context::<AppService>();
    let filtered_keys = Signal::new(BTreeSet::new());
    let ingredients_map = Signal::new(BTreeMap::new());
    let extras = Signal::new(Vec::<(usize, (Signal<String>, Signal<String>))>::new());
    let modified_amts = Signal::new(BTreeMap::new());
    create_effect(cloned!((app_service, ingredients_map) => move || {
        ingredients_map.set(app_service.get_shopping_list());
    }));
    debug!(ingredients_map=?ingredients_map.get_untracked());
    let ingredients = create_memo(cloned!((ingredients_map, filtered_keys) => move || {
        let mut ingredients = Vec::new();
        // This has the effect of sorting the ingredients by category
        for (_, ingredients_list) in ingredients_map.get().iter() {
            for (i, recipes) in ingredients_list.iter() {
                if !filtered_keys.get().contains(&i.key()) {
                    ingredients.push((i.key(), (i.clone(), recipes.clone())));
                }
            }
        }
        ingredients
    }));
    debug!(ingredients = ?ingredients.get_untracked());
    let table_view = Signal::new(View::empty());
    create_effect(
        cloned!((table_view, ingredients, filtered_keys, modified_amts, extras) => move || {
            if (ingredients.get().len() > 0) || (extras.get().len() > 0) {
                let t = view ! {
                    table(class="pad-top shopping-list page-breaker container-fluid", role="grid") {
                        tr {
                            th { " Quantity " }
                            th { " Delete " }
                            th { " Ingredient " }
                            th { " Recipes " }
                        }
                        tbody {
                            Indexed(IndexedProps{
                            iterable: ingredients.clone(),
                            template: cloned!((filtered_keys, modified_amts) => move |(k, (i, rs))| {
                                let mut modified_amt_set = (*modified_amts.get()).clone();
                                let amt = modified_amt_set.entry(k.clone()).or_insert(Signal::new(format!("{}", i.amt.normalize()))).clone();
                                modified_amts.set(modified_amt_set);
                                let name = i.name;
                                let category = if i.category == "" { "other".to_owned() } else { i.category };
                                let form = i.form.map(|form| format!("({})", form)).unwrap_or_default();
                                let recipes = rs.iter().fold(String::new(), |acc, s| format!("{}{},", acc, s)).trim_end_matches(",").to_owned();
                                view! {
                                    tr {
                                        td {
                                            input(bind:value=amt.clone(), type="text")
                                        }
                                        td {
                                            input(type="button", class="no-print destructive", value="X", on:click=cloned!((filtered_keys) => move |_| {
                                                let mut keyset = (*filtered_keys.get()).clone();
                                                keyset.insert(k.clone());
                                                filtered_keys.set(keyset);
                                            }))
                                        }
                                        td {  (name) " " (form) "" br {} "" (category) "" }
                                        td { (recipes) }
                                    }
                                }
                            }),
                        })
                        Indexed(IndexedProps{
                            iterable: extras.handle(),
                            template: cloned!((extras) => move |(idx, (amt, name))| {
                                view! {
                                    tr {
                                        td {
                                            input(bind:value=amt.clone(), type="text")
                                        }
                                        td {
                                            input(type="button", class="no-print destructive", value="X", on:click=cloned!((extras) => move |_| {
                                                extras.set(extras.get().iter()
                                                .filter(|(i, _)| *i != idx)
                                                .map(|(_, v)| v.clone())
                                                .enumerate()
                                                .collect())
                                            }))
                                        }
                                        td {
                                            input(bind:value=name.clone(), type="text")
                                        }
                                        td { "Misc" }
                                    }
                                }
                            })
                        })
                    }
                    }
                };
                table_view.set(t);
            } else {
                table_view.set(View::empty());
            }
        }),
    );
    view! {
        h1 { "Shopping List " }
        (table_view.get().as_ref().clone())
        input(type="button", value="Add Item", class="no-print", on:click=cloned!((extras) => move |_| {
            let mut cloned_extras: Vec<(Signal<String>, Signal<String>)> = (*extras.get()).iter().map(|(_, v)| v.clone()).collect();
            cloned_extras.push((Signal::new("".to_owned()), Signal::new("".to_owned())));
            extras.set(cloned_extras.drain(0..).enumerate().collect());
        }))
        input(type="button", value="Reset", class="no-print", on:click=cloned!((ingredients_map, filtered_keys, app_service, modified_amts, extras) => move |_| {
            // TODO(jwall): We should actually pop up a modal here or use a different set of items.
            ingredients_map.set(app_service.get_shopping_list());
            // clear the filter_signal
            filtered_keys.set(BTreeSet::new());
            modified_amts.set(BTreeMap::new());
            extras.set(Vec::new());
        }))
    }
}
