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
use sycamore::{futures::spawn_local_in_scope, prelude::*};
use tracing::{debug, error};
use web_sys::HtmlDialogElement;

use crate::{js_lib::get_element_by_id, service::get_appservice_from_context};
use recipe_store::RecipeEntry;
use recipes;

fn get_error_dialog() -> HtmlDialogElement {
    get_element_by_id::<HtmlDialogElement>("error-dialog")
        .expect("error-dialog isn't an html dialog element!")
        .unwrap()
}

fn check_recipe_parses(text: &str, error_text: Signal<String>) -> bool {
    if let Err(e) = recipes::parse::as_recipe(text) {
        error!(?e, "Error parsing recipe");
        error_text.set(e);
        let el = get_error_dialog();
        el.show();
        false
    } else {
        error_text.set(String::new());
        let el = get_error_dialog();
        el.close();
        true
    }
}

#[component(Editor<G>)]
fn editor(recipe: RecipeEntry) -> View<G> {
    let id = Signal::new(recipe.recipe_id().to_owned());
    let text = Signal::new(recipe.recipe_text().to_owned());
    let error_text = Signal::new(String::new());
    let app_service = get_appservice_from_context();
    let save_signal = Signal::new(());

    create_effect(
        cloned!((id, app_service, text, save_signal, error_text) => move || {
                // TODO(jwall): This is triggering on load which is not desired.
                save_signal.get();
                spawn_local_in_scope({
                    cloned!((id, app_service, text, error_text) => async move {
                        if let Err(e) = app_service
                            .save_recipes(vec![RecipeEntry(id.get_untracked().as_ref().clone(), text.get_untracked().as_ref().clone())])
                            .await {
                                error!(?e, "Failed to save recipe");
                                error_text.set(format!("{:?}", e));
                            };
                    })
                });
        }),
    );

    let dialog_view = cloned!((error_text) => view! {
        dialog(id="error-dialog") {
            article{
                header {
                    a(href="#", on:click=|_| {
                        let el = get_error_dialog();
                        el.close();
                    }, class="close")
                    "Invalid Recipe"
                }
                p {
                    (error_text.get().clone())
                }
            }
        }
    });

    cloned!((text, error_text) => view! {
        (dialog_view)
        textarea(bind:value=text.clone(), rows=20)
        a(role="button" , href="#", on:click=cloned!((text, error_text) => move |_| {
            let unparsed = text.get();
            check_recipe_parses(unparsed.as_str(), error_text.clone());
        })) { "Check" } " "
        a(role="button", href="#", on:click=cloned!((text, error_text) => move |_| {
            let unparsed = text.get();
            if check_recipe_parses(unparsed.as_str(), error_text.clone()) {
                debug!("triggering a save");
                save_signal.trigger_subscribers();
            };
        })) { "Save" }
    })
}

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
    let show_edit = Signal::new(false);
    create_effect(cloned!((idx, app_service, view, show_edit) => move || {
        if *show_edit.get() {
            return;
        }
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
    create_effect(cloned!((idx, app_service, view, show_edit) => move || {
        let recipe_id: String = idx.get().as_ref().to_owned();
        if !(*show_edit.get()) {
            return;
        }
        if let Some(entry) = app_service.fetch_recipe_text(recipe_id.as_str()).expect("No such recipe") {
            view.set(view! {
                Editor(entry)
            });
        }
    }));
    view! {
        a(role="button", href="#", on:click=cloned!((show_edit) => move |_| { show_edit.set(true); })) { "Edit" } " "
        a(role="button", href="#", on:click=cloned!((show_edit) => move |_| { show_edit.set(false); })) { "View" }
        (view.get().as_ref())
    }
}
