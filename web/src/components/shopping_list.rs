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

use recipes::{Ingredient, IngredientKey};
use sycamore::prelude::*;
use tracing::{debug, instrument};

fn make_ingredients_rows<'ctx, G: Html>(
    cx: Scope<'ctx>,
    ingredients: &'ctx ReadSignal<Vec<(IngredientKey, (Ingredient, BTreeSet<String>))>>,
    modified_amts: &'ctx Signal<BTreeMap<IngredientKey, RcSignal<String>>>,
    filtered_keys: RcSignal<BTreeSet<IngredientKey>>,
) -> View<G> {
    view!(
        cx,
        Indexed(
            iterable = ingredients,
            view = move |cx, (k, (i, rs))| {
                let mut modified_amt_set = modified_amts.get().as_ref().clone();
                let amt = modified_amt_set
                    .entry(k.clone())
                    .or_insert(create_rc_signal(format!("{}", i.amt.normalize())))
                    .clone();
                modified_amts.set(modified_amt_set);
                let name = i.name;
                let category = if i.category == "" {
                    "other".to_owned()
                } else {
                    i.category
                };
                let form = i.form.map(|form| format!("({})", form)).unwrap_or_default();
                let recipes = rs
                    .iter()
                    .fold(String::new(), |acc, s| format!("{}{},", acc, s))
                    .trim_end_matches(",")
                    .to_owned();
                view! {cx,
                    tr {
                        td {
                            input(bind:value=amt, type="text")
                        }
                        td {
                            input(type="button", class="no-print destructive", value="X", on:click={
                                let filtered_keys = filtered_keys.clone();
                                move |_| {
                                let mut keyset = filtered_keys.get().as_ref().clone();
                                keyset.insert(k.clone());
                                filtered_keys.set(keyset);
                            }})
                        }
                        td {  (name) " " (form) "" br {} "" (category) "" }
                        td { (recipes) }
                    }
                }
            }
        )
    )
}

fn make_extras_rows<'ctx, G: Html>(
    cx: Scope<'ctx>,
    extras: &'ctx Signal<Vec<(usize, (&'ctx Signal<String>, &'ctx Signal<String>))>>,
) -> View<G> {
    view! {cx,
                Indexed(
                    iterable=extras,
                    view= move |cx, (idx, (amt, name))| {
                        view! {cx,
                            tr {
                                td {
                                    input(bind:value=amt, type="text")
                                }
                                td {
                                    input(type="button", class="no-print destructive", value="X", on:click=move |_| {
                                        extras.set(extras.get().iter()
                                        .filter(|(i, _)| *i != idx)
                                        .map(|(_, v)| v.clone())
                                        .enumerate()
                                        .collect())
                                    })
                                }
                                td {
                                    input(bind:value=name, type="text")
                                }
                                td { "Misc" }
                            }
                        }
                    }
                )
    }
}

fn make_shopping_table<'ctx, G: Html>(
    cx: Scope<'ctx>,
    ingredients: &'ctx ReadSignal<Vec<(IngredientKey, (Ingredient, BTreeSet<String>))>>,
    modified_amts: &'ctx Signal<BTreeMap<IngredientKey, RcSignal<String>>>,
    extras: &'ctx Signal<Vec<(usize, (&'ctx Signal<String>, &'ctx Signal<String>))>>,
    filtered_keys: RcSignal<BTreeSet<IngredientKey>>,
) -> View<G> {
    let extra_rows_view = make_extras_rows(cx, extras);
    let ingredient_rows =
        make_ingredients_rows(cx, ingredients, modified_amts, filtered_keys.clone());
    view! {cx,
        table(class="pad-top shopping-list page-breaker container-fluid", role="grid") {
            tr {
                th { " Quantity " }
                th { " Delete " }
                th { " Ingredient " }
                th { " Recipes " }
            }
            tbody {
                (ingredient_rows)
                (extra_rows_view)
            }
        }
    }
}

#[instrument]
#[component]
pub fn ShoppingList<G: Html>(cx: Scope) -> View<G> {
    let filtered_keys: RcSignal<BTreeSet<IngredientKey>> = create_rc_signal(BTreeSet::new());
    let ingredients_map = create_rc_signal(BTreeMap::new());
    let extras = create_signal(
        cx,
        Vec::<(usize, (&Signal<String>, &Signal<String>))>::new(),
    );
    let modified_amts = create_signal(cx, BTreeMap::new());
    let show_staples = create_signal(cx, true);
    create_effect(cx, {
        let state = crate::app_state::State::get_from_context(cx);
        let ingredients_map = ingredients_map.clone();
        move || {
            ingredients_map.set(state.get_shopping_list(*show_staples.get()));
        }
    });
    debug!(ingredients_map=?ingredients_map.get_untracked());
    let ingredients = create_memo(cx, {
        let filtered_keys = filtered_keys.clone();
        let ingredients_map = ingredients_map.clone();
        move || {
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
        }
    });
    let table_view = create_signal(cx, View::empty());
    create_effect(cx, {
        let filtered_keys = filtered_keys.clone();
        move || {
            if (ingredients.get().len() > 0) || (extras.get().len() > 0) {
                table_view.set(make_shopping_table(
                    cx,
                    ingredients,
                    modified_amts.clone(),
                    extras.clone(),
                    filtered_keys.clone(),
                ));
            } else {
                table_view.set(View::empty());
            }
        }
    });
    view! {cx,
        h1 { "Shopping List " }
        label(for="show_staples_cb") { "Show staples" }
        input(id="show_staples_cb", type="checkbox", bind:checked=show_staples)
        (table_view.get().as_ref().clone())
        input(type="button", value="Add Item", class="no-print", on:click=move |_| {
            let mut cloned_extras: Vec<(&Signal<String>, &Signal<String>)> = (*extras.get()).iter().map(|(_, tpl)| *tpl).collect();
            cloned_extras.push((create_signal(cx, "".to_owned()), create_signal(cx, "".to_owned())));
            extras.set(cloned_extras.drain(0..).enumerate().collect());
        })
        input(type="button", value="Reset", class="no-print", on:click={
            let state = crate::app_state::State::get_from_context(cx);
            move |_| {
                // TODO(jwall): We should actually pop up a modal here or use a different set of items.
                ingredients_map.set(state.get_shopping_list(*show_staples.get()));
                // clear the filter_signal
                filtered_keys.set(BTreeSet::new());
                modified_amts.set(BTreeMap::new());
                extras.set(Vec::new());
            }
        })
    }
}
