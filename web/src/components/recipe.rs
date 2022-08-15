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
use recipes;
use sycamore::prelude::*;

use crate::service::get_appservice_from_context;

#[component(Steps<G>)]
fn steps(steps: ReadSignal<Vec<recipes::Step>>) -> View<G> {
    view! {
            h2 { "Steps: " }
            div(class="recipe_steps") {
                Indexed(IndexedProps{
                    iterable: steps,
                    template: |step: recipes::Step| { view! {
                        div {
                            h3 { "Instructions" }
                            ul(class="ingredients") {
                                Indexed(IndexedProps{
                                    iterable: Signal::new(step.ingredients).handle(),
                                    template: |i| { view! {
                                        li {
                                            (i.amt) " " (i.name) " " (i.form.as_ref().map(|f| format!("({})", f)).unwrap_or(String::new()))
                                        }
                                    }}
                                })
                            }
                            div(class="instructions") {
                                (step.instructions)
                            }
                        }}
                    }
                })
            }
    }
}

#[component(Recipe<G>)]
pub fn recipe(idx: ReadSignal<String>) -> View<G> {
    let app_service = get_appservice_from_context();
    let view = Signal::new(View::empty());
    create_effect(cloned!((app_service, view) => move || {
        let recipe_id: String = idx.get().as_ref().to_owned();
        if let Some(recipe) = app_service.get_recipes().get().get(&recipe_id) {
            let recipe = recipe.clone();
            let title = create_memo(cloned!((recipe) => move || recipe.get().title.clone()));
            let desc = create_memo(
                cloned!((recipe) => move || recipe.clone().get().desc.clone().unwrap_or_else(|| String::new())),
            );
            let steps = create_memo(cloned!((recipe) => move || recipe.get().steps.clone()));
            view.set(view! {
                div(class="recipe") {
                    h1(class="recipe_title") { (title.get()) }
                     div(class="recipe_description") {
                         (desc.get())
                     }
                    Steps(steps)
                }
            });
        }
    }));
    view! {
        (view.get().as_ref())
    }
}
