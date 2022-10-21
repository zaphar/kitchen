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
use sycamore::{futures::spawn_local_scoped, prelude::*};
use tracing::{error, instrument};

use crate::components::recipe_selection::*;
use crate::{api::*, app_state};

#[allow(non_snake_case)]
#[instrument]
pub fn RecipeSelector<G: Html>(cx: Scope) -> View<G> {
    let rows = create_memo(cx, move || {
        let state = app_state::State::get_from_context(cx);
        let mut rows = Vec::new();
        for row in state
            .recipes
            .get()
            .as_ref()
            .iter()
            .map(|(k, v)| create_signal(cx, (k.clone(), v.clone())))
            .collect::<Vec<&Signal<(String, Recipe)>>>()
            .chunks(4)
        {
            rows.push(create_signal(cx, Vec::from(row)));
        }
        rows
    });
    let clicked = create_signal(cx, false);
    create_effect(cx, move || {
        clicked.track();
        let store = HttpStore::get_from_context(cx);
        let state = app_state::State::get_from_context(cx);
        spawn_local_scoped(cx, {
            async move {
                if let Err(err) = init_page_state(store.as_ref(), state.as_ref()).await {
                    error!(?err);
                };
            }
        });
    });
    view! {cx,
        table(class="recipe_selector no-print") {
            (View::new_fragment(
                rows.get().iter().cloned().map(|r| {
                    view ! {cx,
                        tr { Keyed(
                            iterable=r,
                            view=|cx, sig| {
                                let title = create_memo(cx, move || sig.get().1.title.clone());
                                view! {cx,
                                    td { RecipeSelection(i=sig.get().0.to_owned(), title=title) }
                                }
                            },
                            key=|sig| sig.get().0.to_owned(),
                        )}
                    }
                }).collect()
            ))
        }
        input(type="button", value="Refresh Recipes", on:click=move |_| {
            // Poor man's click event signaling.
            let toggle = !*clicked.get();
            clicked.set(toggle);
        })
    }
}
