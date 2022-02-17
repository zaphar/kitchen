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
use crate::components::Recipe;
use crate::console_log;
use crate::service::AppService;
use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};

use recipes::{Ingredient, IngredientKey};
use sycamore::{context::use_context, prelude::*};

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
    let titles = create_memo(cloned!(app_service => move || {
        let length = app_service.get_recipes().get().len();
        let partition = length/2;
        let col1 = app_service.get_recipes().get().iter().take(partition).map(|(i, r)| (*i, r.clone())).collect::<Vec<(usize, Signal<recipes::Recipe>)>>();
        let col2 = app_service.get_recipes().get().iter().skip(partition).map(|(i, r)| (*i, r.clone())).collect::<Vec<(usize, Signal<recipes::Recipe>)>>();
        (col1, col2)
    }));
    let col1 = create_memo(cloned!((titles) => move || titles.get().0.clone()));
    let col2 = create_memo(cloned!((titles) => move || titles.get().1.clone()));
    view! {
        fieldset(class="recipe_selector no-print container no-left-mgn") {
            div(class="row") {Indexed(IndexedProps{
                iterable: col1,
                template: |(i, recipe)| {
                    view! {
                        RecipeSelection(RecipeCheckBoxProps{i: i, title: create_memo(move || recipe.get().title.clone())})
                    }
                },
            })}
            div(class="row") {Indexed(IndexedProps{
                iterable: col2,
                template: |(i, recipe)| {
                    view! {
                        RecipeSelection(RecipeCheckBoxProps{i: i, title: create_memo(move || recipe.get().title.clone())})
                    }
                },
            })}
        }
    }
}

#[component(ShoppingList<G>)]
fn shopping_list() -> View<G> {
    let app_service = use_context::<AppService>();
    let filtered_keys = Signal::new(HashSet::new());
    let ingredients_map = Signal::new(BTreeMap::new());
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
        cloned!((table_view, ingredients, filtered_keys) => move || {
            if ingredients.get().len() > 0 {
                let t = view ! {
                    table(class="shopping-list page-breaker table table-striped table-condensed table-responsive") {
                        tr {
                            th { " Quantity " }
                            th { " Ingredient " }
                        }
                        tbody {Indexed(IndexedProps{
                            iterable: ingredients.clone(),
                            template: cloned!((filtered_keys) => move |(k, i)| {
                                let amt = Signal::new(format!("{}", i.amt.normalize()));
                                // TODO(jwall): Create effect to reset this amount if it diverges.
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
        h1 { "Shopping List " input(type="button", value="Reset", class="no-print", on:click=cloned!((ingredients_map, filtered_keys, app_service) => move |_| {
            // trigger the shopping list generation
            ingredients_map.set(app_service.get_shopping_list());
            // clear the filter_signal
            filtered_keys.set(HashSet::new());
        }))}

        (table_view.get().as_ref().clone())
    }
}

#[component(RecipeList<G>)]
fn recipe_list() -> View<G> {
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

#[component(MealPlan<G>)]
pub fn meal_plan() -> View<G> {
    view! {
        h1 {
            "Select your recipes"
        }
        RecipeSelector()
        ShoppingList()
        RecipeList()
    }
}
