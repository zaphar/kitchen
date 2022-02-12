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
use crate::console_log;
use crate::service::AppService;
use std::rc::Rc;

use recipes::{Ingredient, IngredientKey};
use sycamore::{context::use_context, prelude::*};

struct RecipeCheckBoxProps {
    i: usize,
    title: String,
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
        input(type="number", min="0", bind:value=count.clone(), name=format!("recipe_id:{}", i), value=id_as_str.clone(), on:change=move |_| {
            let mut app_service = app_service.clone();
            console_log!("setting recipe id: {} to count: {}", i, *count.get());
            app_service.set_recipe_count_by_index(i, count.get().parse().unwrap());
        })
        label(for=id_cloned_2) { (props.title) }
    }
}

#[component(RecipeSelector<G>)]
pub fn recipe_selector() -> View<G> {
    let app_service = use_context::<AppService>();
    let titles = create_memo(cloned!(app_service => move || {
        app_service.get_recipes().get().iter().map(|(i, r)| (*i, r.title.clone())).collect::<Vec<(usize, String)>>()
    }));
    view! {
        fieldset(class="recipe_selector") {
            Keyed(KeyedProps{
                iterable: titles,
                template: |(i, title)| {
                    view! {
                        RecipeSelection(RecipeCheckBoxProps{i: i, title: title})
                    }
                },
                key: |(i, title)| (*i, title.clone()),
            })
        }
    }
}

#[component(ShoppingList<G>)]
fn shopping_list() -> View<G> {
    let app_service = use_context::<AppService>();
    let ingredients = create_memo(move || {
        let ingredients = app_service.get_menu_list();
        ingredients
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(IngredientKey, Ingredient)>>()
    });

    // TODO(jwall): Sort by categories and names.
    view! {
        table(class="shopping_list") {
            tr {
                th { "Quantity" }
                th { "Ingredient" }
            }
            Indexed(IndexedProps{
                iterable: ingredients,
                template: |(_k, i)| {
                    let amt = Signal::new(format!("{}", i.amt.normalize()));
                    view! {
                        tr {
                            td { input(bind:value=amt.clone(), type="text") }
                            td { (i.name) }
                        }
                    }
                },
            })
        }
    }
}

#[component(ShoppingView<G>)]
pub fn shopping_view() -> View<G> {
    view! {
        h1 {
            "Select your recipes"
        }
        RecipeSelector()
        ShoppingList()
    }
}
