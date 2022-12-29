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
use tracing::{info, instrument};

use crate::app_state::{Message, StateHandler};

fn make_ingredients_rows<'ctx, G: Html>(
    cx: Scope<'ctx>,
    sh: StateHandler<'ctx>,
    show_staples: &'ctx ReadSignal<bool>,
) -> View<G> {
    let ingredients = sh.get_selector(cx, move |state| {
        let state = state.get();
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
                acc.accumulate_from(staples);
            }
        }
        acc.ingredients()
            .into_iter()
            // First we filter out any filtered ingredients
            .filter(|(i, _)| state.filtered_ingredients.contains(i))
            // Then we take into account our modified amts
            .map(|(k, (i, rs))| {
                if state.modified_amts.contains_key(&k) {
                    (
                        k.clone(),
                        (
                            i.name,
                            i.form,
                            i.category,
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
                            i.category,
                            format!("{}", i.amt.normalize()),
                            rs,
                        ),
                    )
                }
            })
            .collect::<Vec<(
                IngredientKey,
                (String, Option<String>, String, String, BTreeSet<String>),
            )>>()
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
                sh.bind_trigger(cx, &amt_signal, move |val| {
                    Message::UpdateAmt(k_clone.clone(), val.as_ref().clone())
                });
                let form = form.map(|form| format!("({})", form)).unwrap_or_default();
                let recipes = rs
                    .iter()
                    .fold(String::new(), |acc, s| format!("{}{},", acc, s))
                    .trim_end_matches(",")
                    .to_owned();
                view! {cx,
                    tr {
                        td {
                            input(bind:value=amt_signal, type="text")
                        }
                        td {
                            input(type="button", class="no-print destructive", value="X", on:click={
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

fn make_extras_rows<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let extras_read_signal = sh.get_selector(cx, |state| {
        state
            .get()
            .extras
            .iter()
            .cloned()
            .collect::<Vec<(String, String)>>()
    });
    view! {cx,
                Indexed(
                    iterable=extras_read_signal,
                    view= move |cx, (amt, name)| {
                        let amt_signal = create_signal(cx, amt.clone());
                        let name_signal = create_signal(cx, name.clone());
                        create_effect(cx, {
                            let amt_clone = amt.clone();
                            let name_clone = name.clone();
                            move || {
                                let new_amt = amt_signal.get();
                                let new_name = name_signal.get();
                                sh.dispatch(cx, Message::RemoveExtra(amt_clone.clone(), name_clone.clone()));
                                sh.dispatch(cx, Message::AddExtra(new_amt.as_ref().clone(), new_name.as_ref().clone()));
                            }
                        });
                        view! {cx,
                            tr {
                                td {
                                    input(bind:value=amt_signal, type="text")
                                }
                                td {
                                    input(type="button", class="no-print destructive", value="X", on:click={
                                        move |_| {
                                            sh.dispatch(cx, Message::RemoveExtra(amt.clone(), name.clone()));
                                    }})
                                }
                                td {
                                    input(bind:value=name_signal, type="text")
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
    let extra_rows_view = make_extras_rows(cx, sh);
    let ingredient_rows = make_ingredients_rows(cx, sh, show_staples);
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

#[instrument(skip_all)]
#[component]
pub fn ShoppingList<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let show_staples = create_signal(cx, true);
    view! {cx,
        h1 { "Shopping List " }
        label(for="show_staples_cb") { "Show staples" }
        input(id="show_staples_cb", type="checkbox", bind:checked=show_staples)
        (make_shopping_table(cx, sh, show_staples))
        input(type="button", value="Add Item", class="no-print", on:click=move |_| {
            sh.dispatch(cx, Message::AddExtra(String::new(), String::new()));
        })
        input(type="button", value="Reset", class="no-print", on:click=move |_| {
                sh.dispatch(cx, Message::ResetInventory);
        })
        input(type="button", value="Save", class="no-print", on:click=move |_| {
        info!("Registering save request for inventory");
            sh.dispatch(cx, Message::SaveState);
        })
    }
}
