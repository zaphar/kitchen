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
use sycamore::{futures::spawn_local_in_scope, prelude::*};
use tracing::{error, instrument};

use crate::components::recipe_selection::*;
use crate::service::get_appservice_from_context;

#[instrument]
#[component(RecipeSelector<G>)]
pub fn recipe_selector() -> View<G> {
    let app_service = get_appservice_from_context();
    let rows = create_memo(cloned!(app_service => move || {
        let mut rows = Vec::new();
        for row in app_service.get_recipes().get().iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<(String, Signal<Recipe>)>>().chunks(4) {
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
                if let Err(err) = app_service.refresh().await {
                    error!(?err);
                };
            }
        }));
    }));
    view! {
        table(class="recipe_selector no-print") {
            (View::new_fragment(
                rows.get().iter().cloned().map(|r| {
                    view ! {
                        tr { Keyed(KeyedProps{
                            iterable: r.handle(),
                            template: |(i, recipe)| {
                                view! {
                                    td { RecipeSelection(RecipeCheckBoxProps{i: i, title: create_memo(move || recipe.get().title.clone())}) }
                                }
                            },
                            key: |r| r.0.clone(),
                        })}
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
