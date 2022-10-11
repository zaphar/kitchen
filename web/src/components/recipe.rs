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
use sycamore::{futures::spawn_local_scoped, prelude::*};
use tracing::{debug, error};
use web_sys::HtmlDialogElement;

use crate::{js_lib::get_element_by_id, service::AppService};
use recipe_store::RecipeEntry;
use recipes;

fn get_error_dialog() -> HtmlDialogElement {
    get_element_by_id::<HtmlDialogElement>("error-dialog")
        .expect("error-dialog isn't an html dialog element!")
        .expect("error-dialog element isn't present")
}

fn check_recipe_parses(text: &str, error_text: &Signal<String>) -> bool {
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

#[component]
fn Editor<G: Html>(cx: Scope, recipe: &RecipeEntry) -> View<G> {
    let id = create_signal(cx, recipe.recipe_id().to_owned());
    let text = create_signal(cx, recipe.recipe_text().to_owned());
    let error_text = create_signal(cx, String::new());
    let app_service = use_context::<AppService>(cx);
    let save_signal = create_signal(cx, ());

    create_effect(cx, move || {
        // TODO(jwall): This is triggering on load which is not desired.
        save_signal.track();
        spawn_local_scoped(cx, {
            async move {
                if let Err(e) = app_service
                    .save_recipes(vec![RecipeEntry(
                        id.get_untracked().as_ref().clone(),
                        text.get_untracked().as_ref().clone(),
                    )])
                    .await
                {
                    error!(?e, "Failed to save recipe");
                    error_text.set(format!("{:?}", e));
                };
            }
        });
    });

    let dialog_view = view! {cx,
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
    };

    view! {cx,
        (dialog_view)
        textarea(bind:value=text, rows=20)
        a(role="button" , href="#", on:click=move |_| {
            let unparsed = text.get();
            check_recipe_parses(unparsed.as_str(), error_text.clone());
        }) { "Check" } " "
        a(role="button", href="#", on:click=move |_| {
            let unparsed = text.get();
            if check_recipe_parses(unparsed.as_str(), error_text.clone()) {
                debug!("triggering a save");
                save_signal.trigger_subscribers();
            };
        }) { "Save" }
    }
}

#[component]
fn Steps<'ctx, G: Html>(cx: Scope<'ctx>, steps: &'ctx ReadSignal<Vec<recipes::Step>>) -> View<G> {
    view! {cx,
            h2 { "Steps: " }
            div(class="recipe_steps") {
                Indexed(
                    iterable=steps,
                    view = |cx, step: recipes::Step| { view! {cx,
                        div {
                            h3 { "Instructions" }
                            ul(class="ingredients") {
                                Indexed(
                                    iterable = create_signal(cx, step.ingredients),
                                    view = |cx, i| { view! {cx,
                                        li {
                                            (i.amt) " " (i.name) " " (i.form.as_ref().map(|f| format!("({})", f)).unwrap_or(String::new()))
                                        }
                                    }}
                                )
                            }
                            div(class="instructions") {
                                (step.instructions)
                            }
                        }}
                    }
                )
            }
    }
}

#[component]
pub fn Recipe<'ctx, G: Html>(cx: Scope<'ctx>, recipe_id: String) -> View<G> {
    let app_service = use_context::<AppService>(cx).clone();
    let view = create_signal(cx, View::empty());
    let show_edit = create_signal(cx, false);
    // FIXME(jwall): This has too many unwrap() calls
    if let Some(recipe) = app_service
        .fetch_recipes_from_storage()
        .expect("Failed to fetch recipes from storage")
        .1
        .expect(&format!("No recipe counts for recipe id: {}", recipe_id))
        .get(&recipe_id)
    {
        let recipe = create_signal(cx, recipe.clone());
        let title = create_memo(cx, move || recipe.get().title.clone());
        let desc = create_memo(cx, move || {
            recipe
                .clone()
                .get()
                .desc
                .clone()
                .unwrap_or_else(|| String::new())
        });
        let steps = create_memo(cx, move || recipe.get().steps.clone());
        create_effect(cx, move || {
            if *show_edit.get() {
                return;
            }
            view.set(view! {cx,
                div(class="recipe") {
                    h1(class="recipe_title") { (title.get()) }
                     div(class="recipe_description") {
                         (desc.get())
                     }
                    Steps(steps)
                }
            });
        });
        if let Some(entry) = app_service
            .fetch_recipe_text(recipe_id.as_str())
            .expect("No such recipe")
        {
            let entry_ref = create_ref(cx, entry);
            create_effect(cx, move || {
                if !(*show_edit.get()) {
                    return;
                }
                view.set(view! {cx,
                    Editor(entry_ref)
                });
            });
        }
    }
    view! {cx,
        a(role="button", href="#", on:click=move |_| { show_edit.set(true); }) { "Edit" } " "
        a(role="button", href="#", on:click=move |_| { show_edit.set(false); }) { "View" }
        (view.get().as_ref())
    }
}
