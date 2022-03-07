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
use crate::{components::Recipe, service::AppService};
use crate::{console_error, console_log};
use std::collections::HashMap;
use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};

use recipes::{Ingredient, IngredientKey};
use sycamore::{context::use_context, futures::spawn_local_in_scope, prelude::*};

struct RecipeCheckBoxProps {
    i: usize,
    title: ReadSignal<String>,
}

#[component(RecipeSelection<G>)]
fn recipe_selection(props: RecipeCheckBoxProps) -> View<G> {
    let app_service = use_context::<AppService>();
    // This is total hack but it works around the borrow issues with
    // the `view!` macro.
    let i = props.i;
    let id_as_str = Rc::new(format!("{}", i));
    let id_cloned_2 = id_as_str.clone();
    let count = Signal::new(format!("{}", app_service.get_recipe_count_by_index(i)));
    view! {
        div(class="form-group col-md-1") {
            input(type="number", class="item-count-sel", min="0", bind:value=count.clone(), name=format!("recipe_id:{}", i), value=id_as_str.clone(), on:change=move |_| {
                let mut app_service = app_service.clone();
                console_log!("setting recipe id: {} to count: {}", i, *count.get());
                app_service.set_recipe_count_by_index(i, count.get().parse().unwrap());
            })
            label(for=id_cloned_2) { (props.title.get()) }
        }
    }
}

#[component(RecipeSelector<G>)]
pub fn recipe_selector() -> View<G> {
    let app_service = use_context::<AppService>();
    let rows = create_memo(cloned!(app_service => move || {
        let mut rows = Vec::new();
        for row in app_service.get_recipes().get().as_slice().chunks(4) {
            rows.push(Signal::new(Vec::from(row)));
        }
        rows
    }));
    let clicked = Signal::new(false);
    create_effect(cloned!((clicked, app_service) => move || {
        clicked.get();
        spawn_local_in_scope(cloned!((app_service) => {
            let mut app_service = app_service.clone();
            async move {
                if let Err(e) = app_service.refresh_recipes().await {
                    console_error!("{}", e);
                };
            }
        }));
    }));
    view! {
        input(type="button", value="Refresh Recipes", on:click=move |_| {
            // Poor man's click event signaling.
            let toggle = !*clicked.get();
            clicked.set(toggle);
        })
        fieldset(class="recipe_selector no-print container no-left-mgn pad-top") {
            (View::new_fragment(
                rows.get().iter().cloned().map(|r| {
                    view ! {
                        div(class="row") {Indexed(IndexedProps{
                            iterable: r.handle(),
                            template: |(i, recipe)| {
                                view! {
                                    RecipeSelection(RecipeCheckBoxProps{i: i, title: create_memo(move || recipe.get().title.clone())})
                                }
                            },
                        })}
                    }
                }).collect()
            ))
        }
    }
}

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

#[component(RecipeList<G>)]
pub fn recipe_list() -> View<G> {
    let app_service = use_context::<AppService>();
    let menu_list = create_memo(move || app_service.get_menu_list());
    view! {
        h1 { "Recipe List" }
        Indexed(IndexedProps{
            iterable: menu_list,
            template: |(idx, _count)| {
                console_log!("Rendering recipe index: {}", idx);
                let idx = Signal::new(idx);
                view ! {
                    Recipe(idx.handle())
                    hr()
                }
            }
        })
    }
}
