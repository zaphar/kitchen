use std::collections::BTreeMap;

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
use recipes::Recipe;
use sycamore::prelude::*;
use tracing::{debug, instrument};

use crate::app_state::{Message, StateHandler};
use crate::components::recipe_selection::*;

#[derive(Props)]
pub struct CategoryGroupProps<'ctx> {
    sh: StateHandler<'ctx>,
    category: String,
    recipes: Vec<(String, Recipe)>,
    row_size: usize,
}

#[allow(non_snake_case)]
pub fn CategoryGroup<'ctx, G: Html>(
    cx: Scope<'ctx>,
    CategoryGroupProps {
        sh,
        category,
        recipes,
        row_size,
    }: CategoryGroupProps<'ctx>,
) -> View<G> {
    let rows = create_signal(cx, {
        let mut rows = Vec::new();
        for row in recipes
            .iter()
            .map(|(id, r)| create_signal(cx, (id.clone(), r.clone())))
            .collect::<Vec<&Signal<(String, Recipe)>>>()
            .chunks(row_size)
        {
            rows.push(create_signal(cx, Vec::from(row)));
        }
        rows
    });
    view! {cx,
        h2 { (category) }
        div(class="no-print flex-wrap-start align-stretch") {
            (View::new_fragment(
                rows.get().iter().cloned().map(|r| {
                    view ! {cx,
                        Keyed(
                            iterable=r,
                            view=move |cx, sig| {
                                let title = create_memo(cx, move || sig.get().1.title.clone());
                                view! {cx,
                                    div(class="cell column-flex justify-end align-stretch") { RecipeSelection(i=sig.get().0.to_owned(), title=title, sh=sh) }
                                }
                            },
                            key=|sig| sig.get().0.to_owned(),
                        )
                    }
                }).collect()
            ))
        }
    }
}

#[allow(non_snake_case)]
#[instrument(skip_all)]
pub fn RecipePlan<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let recipe_category_groups = sh.get_selector(cx, |state| {
        state
            .get()
            .recipe_categories
            .iter()
            .fold(BTreeMap::new(), |mut map, (r, cat)| {
                debug!(?cat, recipe_id=?r, "Accumulating recipe into category");
                map.entry(cat.clone()).or_insert(Vec::new()).push((
                    r.clone(),
                    state
                        .get()
                        .recipes
                        .get(r)
                        .expect(&format!("Failed to find recipe {}", r))
                        .clone(),
                ));
                map
            })
            .iter()
            .map(|(cat, rs)| (cat.clone(), rs.clone()))
            .collect::<Vec<(String, Vec<(String, Recipe)>)>>()
    });
    view! {cx,
        Keyed(
            iterable=recipe_category_groups,
            view=move |cx, (cat, recipes)| {
                view! {cx,
                    CategoryGroup(sh=sh, category=cat, recipes=recipes, row_size=4)
                }
            },
            key=|(ref cat, _)| cat.clone(),
        )
        button(on:click=move |_| {
            sh.dispatch(cx, Message::LoadState(None));
        }) { "Reset" } " "
        button(on:click=move |_| {
            sh.dispatch(cx, Message::ResetRecipeCounts);
        }) { "Clear All" } " "
        button(on:click=move |_| {
            // Poor man's click event signaling.
            sh.dispatch(cx, Message::SaveState(None));
        }) { "Save Plan" } " "
    }
}
