// Copyright 2022 Jeremy Wall (Jeremy@marzhilsltudios.com)
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

use crate::app_state::{Message, StateHandler};
use sycamore::prelude::*;
use tracing::instrument;

#[derive(Props)]
struct CategoryRowProps<'ctx> {
    sh: StateHandler<'ctx>,
    ingredient: String,
    category: String,
    ingredient_recipe_map: &'ctx ReadSignal<BTreeMap<String, BTreeSet<String>>>,
}

#[instrument(skip_all)]
#[component]
fn CategoryRow<'ctx, G: Html>(cx: Scope<'ctx>, props: CategoryRowProps<'ctx>) -> View<G> {
    let CategoryRowProps {
        sh,
        ingredient,
        category,
        ingredient_recipe_map,
    } = props;
    let category = create_signal(cx, category);
    let ingredient_clone = ingredient.clone();
    let ingredient_clone2 = ingredient.clone();
    let recipes = create_memo(cx, move || {
        ingredient_recipe_map
            .get()
            .get(&ingredient_clone2)
            .cloned()
            .unwrap_or_else(|| BTreeSet::new())
            .iter()
            .cloned()
            .collect::<Vec<String>>()
    });
    view! {cx,
        tr() {
            td() {
                (ingredient_clone) br()
                Indexed(
                    iterable=recipes,
                    view=|cx, r| {
                        let recipe_name = r.clone();
                        let href = if recipe_name == "Staples" {
                            "/ui/manage/staples".to_owned()
                        } else {
                            format!("/ui/recipe/edit/{}", r)
                        };
                        view!{cx,
                            a(href=href) { (recipe_name) } br()
                        }
                    }
                )
            }
            td() { input(type="text", list="category_options", bind:value=category, on:change={
                let ingredient_clone = ingredient.clone();
                move |_| {
                    sh.dispatch(cx, Message::UpdateCategory(ingredient_clone.clone(), category.get_untracked().as_ref().clone(), None));
                }
            }) }
        }
    }
}

#[instrument(skip_all)]
#[component]
pub fn Categories<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let category_list = sh.get_selector(cx, |state| {
        let mut categories = state
            .get()
            .category_map
            .iter()
            .map(|(_, v)| v.clone())
            .collect::<Vec<String>>();
        categories.sort();
        categories.dedup();
        categories
    });

    let ingredient_recipe_map = sh.get_selector(cx, |state| {
        let state = state.get();
        let mut ingredients: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (recipe_id, r) in state.recipes.iter() {
            for (_, i) in r.get_ingredients().iter() {
                let ingredient_name = i.name.clone();
                ingredients
                    .entry(ingredient_name)
                    .or_insert(BTreeSet::new())
                    .insert(recipe_id.clone());
            }
        }
        if let Some(staples) = &state.staples {
            for i in staples.iter() {
                let ingredient_name = i.name.clone();
                ingredients
                    .entry(ingredient_name)
                    .or_insert(BTreeSet::new())
                    .insert("Staples".to_owned());
            }
        }
        ingredients
    });

    let rows = sh.get_selector(cx, |state| {
        let state = state.get();
        let category_map = state.category_map.clone();
        let mut ingredients = BTreeSet::new();
        for (_, r) in state.recipes.iter() {
            for (_, i) in r.get_ingredients().iter() {
                ingredients.insert(i.name.clone());
            }
        }
        if let Some(staples) = &state.staples {
            for i in staples.iter() {
                ingredients.insert(i.name.clone());
            }
        }
        let mut mapping_list = Vec::new();
        for i in ingredients.iter() {
            let cat = category_map
                .get(i)
                .map(|v| v.clone())
                .unwrap_or_else(|| "None".to_owned());
            mapping_list.push((i.clone(), cat));
        }
        mapping_list.sort_by(|tpl1, tpl2| tpl1.1.cmp(&tpl2.1));
        mapping_list
    });
    view! {cx,
        table() {
            tr {
                th { "Ingredient" }
                th { "Category" }
            }
            Keyed(
                iterable=rows,
                view=move |cx, (i, c)| {
                    view! {cx, CategoryRow(sh=sh, ingredient=i, category=c, ingredient_recipe_map=ingredient_recipe_map)}
                },
                key=|(i, _)| i.clone()
            )
        }
        datalist(id="category_options") {
            Keyed(
                iterable=category_list,
                view=move |cx, c| {
                    view!{cx,
                        option(value=c)
                    }
                },
                key=|c| c.clone(),
            )
        }
    }
}
