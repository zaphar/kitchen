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
use tracing::instrument;

use crate::app_state;
use crate::app_state::{Message, StateHandler};
use crate::components::recipe_selection::*;

#[allow(non_snake_case)]
#[instrument(skip_all)]
pub fn RecipePlan<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let rows = sh.get_selector(cx, move |state| {
        let mut rows = Vec::new();
        for row in state
            .get()
            .recipes
            .iter()
            .map(|(k, v)| create_signal(cx, (k.clone(), v.clone())))
            .collect::<Vec<&Signal<(String, Recipe)>>>()
            .chunks(4)
        {
            rows.push(create_signal(cx, Vec::from(row)));
        }
        rows
    });
    let refresh_click = create_signal(cx, false);
    let save_click = create_signal(cx, false);
    // FIXME(jwall): We should probably make this a dispatch method instead.
    create_effect(cx, move || {
        refresh_click.track();
        sh.dispatch(cx, Message::LoadState);
    });
    create_effect(cx, move || {
        save_click.track();
        sh.dispatch(cx, Message::SaveState);
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
        input(type="button", value="Reset", on:click=move |_| {
            // Poor man's click event signaling.
            let toggle = !*refresh_click.get();
            refresh_click.set(toggle);
        })
        input(type="button", value="Clear All", on:click=move |_| {
            let state = app_state::State::get_from_context(cx);
            state.reset_recipe_counts();
        })
        input(type="button", value="Save Plan", on:click=move |_| {
            // Poor man's click event signaling.
            let toggle = !*save_click.get();
            save_click.set(toggle);
        })
    }
}
