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
use std::collections::BTreeSet;

use recipes::{IngredientAccumulator, IngredientKey};
use sycamore::prelude::*;
use tracing::{debug, info, instrument};

use crate::app_state::{Message, StateHandler};

#[instrument(skip_all)]
fn make_deleted_ingredients_rows<'ctx, G: Html>(
    cx: Scope<'ctx>,
    sh: StateHandler<'ctx>,
    show_staples: &'ctx ReadSignal<bool>,
) -> View<G> {
    debug!("Making ingredients rows");
    let ingredients = sh.get_selector(cx, move |state| {
        let state = state.get();
        let category_map = &state.category_map;
        debug!("building ingredient list from state");
        let mut acc = IngredientAccumulator::new();
        for (id, count) in state.recipe_counts.iter() {
            for _ in 0..(*count) {
                acc.accumulate_from(
                    state
                        .recipes
                        .get(id)
                        .expect(&format!("No such recipe id exists: {}", id)),
                );
            }
        }
        if *show_staples.get() {
            if let Some(staples) = &state.staples {
                acc.accumulate_ingredients_for("Staples", staples.iter());
            }
        }
        let mut ingredients = acc
            .ingredients()
            .into_iter()
            // First we filter out any filtered ingredients
            .filter(|(i, _)| state.filtered_ingredients.contains(i))
            // Then we take into account our modified amts
            .map(|(k, (i, rs))| {
                let category = category_map
                    .get(&i.name)
                    .cloned()
                    .unwrap_or_else(|| String::new());
                if state.modified_amts.contains_key(&k) {
                    (
                        k.clone(),
                        (
                            i.name,
                            i.form,
                            category,
                            state.modified_amts.get(&k).unwrap().clone(),
                            rs,
                        ),
                    )
                } else {
                    (
                        k.clone(),
                        (
                            i.name,
                            i.form,
                            category,
                            format!("{}", i.amt.normalize()),
                            rs,
                        ),
                    )
                }
            })
            .collect::<Vec<(
                IngredientKey,
                (String, Option<String>, String, String, BTreeSet<String>),
            )>>();
        ingredients.sort_by(|tpl1, tpl2| (&tpl1.1 .2, &tpl1.1 .0).cmp(&(&tpl2.1 .2, &tpl2.1 .0)));
        ingredients
    });
    view!(
        cx,
        Indexed(
            iterable = ingredients,
            view = move |cx, (k, (name, form, category, amt, rs))| {
                let category = if category == "" {
                    "other".to_owned()
                } else {
                    category
                };
                let amt_signal = create_signal(cx, amt);
                let k_clone = k.clone();
                let form = form.map(|form| format!("({})", form)).unwrap_or_default();
                let recipes = rs
                    .iter()
                    .fold(String::new(), |acc, s| format!("{}{},", acc, s))
                    .trim_end_matches(",")
                    .to_owned();
                view! {cx,
                    tr {
                        td {
                            input(bind:value=amt_signal, class="width-5", type="text", on:change=move |_| {
                                sh.dispatch(cx, Message::UpdateAmt(k_clone.clone(), amt_signal.get_untracked().as_ref().clone()));
                            })
                        }
                        td {
                            input(type="button", class="fit-content no-print", value="Undo", on:click={
                                move |_| {
                                    sh.dispatch(cx, Message::RemoveFilteredIngredient(k.clone()));
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

#[instrument(skip_all)]
fn make_ingredients_rows<'ctx, G: Html>(
    cx: Scope<'ctx>,
    sh: StateHandler<'ctx>,
    show_staples: &'ctx ReadSignal<bool>,
) -> View<G> {
    debug!("Making ingredients rows");
    let ingredients = sh.get_selector(cx, move |state| {
        let state = state.get();
        let category_map = &state.category_map;
        debug!("building ingredient list from state");
        let mut acc = IngredientAccumulator::new();
        for (id, count) in state.recipe_counts.iter() {
            for _ in 0..(*count) {
                acc.accumulate_from(
                    state
                        .recipes
                        .get(id)
                        .expect(&format!("No such recipe id exists: {}", id)),
                );
            }
        }
        if *show_staples.get() {
            if let Some(staples) = &state.staples {
                acc.accumulate_ingredients_for("Staples", staples.iter());
            }
        }
        let mut ingredients = acc
            .ingredients()
            .into_iter()
            // First we filter out any filtered ingredients
            .filter(|(i, _)| !state.filtered_ingredients.contains(i))
            // Then we take into account our modified amts
            .map(|(k, (i, rs))| {
                let category = category_map
                    .get(&i.name)
                    .cloned()
                    .unwrap_or_else(|| String::new());
                if state.modified_amts.contains_key(&k) {
                    (
                        k.clone(),
                        (
                            i.name,
                            i.form,
                            category,
                            state.modified_amts.get(&k).unwrap().clone(),
                            rs,
                        ),
                    )
                } else {
                    (
                        k.clone(),
                        (
                            i.name,
                            i.form,
                            category,
                            format!("{}", i.amt.normalize()),
                            rs,
                        ),
                    )
                }
            })
            .collect::<Vec<(
                IngredientKey,
                (String, Option<String>, String, String, BTreeSet<String>),
            )>>();
        ingredients.sort_by(|tpl1, tpl2| (&tpl1.1 .2, &tpl1.1 .0).cmp(&(&tpl2.1 .2, &tpl2.1 .0)));
        ingredients
    });
    view!(
        cx,
        Indexed(
            iterable = ingredients,
            view = move |cx, (k, (name, form, category, amt, rs))| {
                let category = if category == "" {
                    "other".to_owned()
                } else {
                    category
                };
                let amt_signal = create_signal(cx, amt);
                let k_clone = k.clone();
                let form = form.map(|form| format!("({})", form)).unwrap_or_default();
                let recipes = rs
                    .iter()
                    .fold(String::new(), |acc, s| format!("{}{},", acc, s))
                    .trim_end_matches(",")
                    .to_owned();
                view! {cx,
                    tr {
                        td {
                            input(bind:value=amt_signal, class="width-5", type="text", on:change=move |_| {
                                sh.dispatch(cx, Message::UpdateAmt(k_clone.clone(), amt_signal.get_untracked().as_ref().clone()));
                            })
                        }
                        td {
                            input(type="button", class="fit-content no-print destructive", value="X", on:click={
                                move |_| {
                                    sh.dispatch(cx, Message::AddFilteredIngredient(k.clone()));
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

#[instrument(skip_all)]
fn make_extras_rows<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    debug!("Making extras rows");
    let extras_read_signal = sh.get_selector(cx, |state| {
        state.get().extras.iter().cloned().enumerate().collect()
    });
    view! {cx,
        Indexed(
            iterable=extras_read_signal,
            view= move |cx, (idx, (amt, name))| {
                let amt_signal = create_signal(cx, amt.clone());
                let name_signal = create_signal(cx, name.clone());
                view! {cx,
                    tr {
                        td {
                            input(bind:value=amt_signal, class="width-5", type="text", on:change=move |_| {
                                sh.dispatch(cx, Message::UpdateExtra(idx,
                                    amt_signal.get_untracked().as_ref().clone(),
                                    name_signal.get_untracked().as_ref().clone()));
                            })
                        }
                        td {
                            input(type="button", class="fit-content no-print destructive", value="X", on:click=move |_| {
                                sh.dispatch(cx, Message::RemoveExtra(idx));
                            })
                        }
                        td {
                            input(bind:value=name_signal, type="text", on:change=move |_| {
                                sh.dispatch(cx, Message::UpdateExtra(idx,
                                    amt_signal.get_untracked().as_ref().clone(),
                                    name_signal.get_untracked().as_ref().clone()));
                            })
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
    sh: StateHandler<'ctx>,
    show_staples: &'ctx ReadSignal<bool>,
) -> View<G> {
    debug!("Making shopping table");
    view! {cx,
        table(class="pad-top shopping-list page-breaker container-fluid", role="grid") {
            tr {
                th { " Quantity " }
                th { " Delete " }
                th { " Ingredient " }
                th { " Recipes " }
            }
            tbody {
                (make_ingredients_rows(cx, sh, show_staples))
                (make_extras_rows(cx, sh))
            }
        }
        ("Deleted Items")
        table(class="pad-top shopping-list page-breaker container-fluid", role="grid") {
            tr {
                th { " Quantity " }
                th { " Delete " }
                th { " Ingredient " }
                th { " Recipes " }
            }
            tbody {
                (make_deleted_ingredients_rows(cx, sh, show_staples))
            }
        }
    }
}

#[instrument(skip_all)]
#[component]
pub fn ShoppingList<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let show_staples = sh.get_selector(cx, |state| state.get().use_staples);
    view! {cx,
        h1 { "Shopping List " }
        label(for="show_staples_cb") { "Show staples" }
        input(id="show_staples_cb", type="checkbox", checked=*show_staples.get(), on:change=move|_| {
            let value = !*show_staples.get_untracked();
            sh.dispatch(cx, Message::UpdateUseStaples(value));
        })
        (make_shopping_table(cx, sh, show_staples))
        button(class="no-print", on:click=move |_| {
            info!("Registering add item request for inventory");
            sh.dispatch(cx, Message::AddExtra(String::new(), String::new()));
        }) { "Add Item" } " "
        button(class="no-print", on:click=move |_| {
            info!("Registering reset request for inventory");
            sh.dispatch(cx, Message::ResetInventory);
        }) { "Reset" } " "
        button(class="no-print", on:click=move |_| {
            info!("Registering save request for inventory");
            sh.dispatch(cx, Message::SaveState(None));
        }) { "Save" } " "
    }
}
