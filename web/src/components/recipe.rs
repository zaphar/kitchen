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

use crate::{
    app_state::{Message, StateHandler},
    js_lib,
};
use recipes::{self, RecipeEntry};

fn check_recipe_parses(
    text: &str,
    error_text: &Signal<String>,
    aria_hint: &Signal<&'static str>,
) -> bool {
    if let Err(e) = recipes::parse::as_recipe(text) {
        error!(?e, "Error parsing recipe");
        error_text.set(e);
        aria_hint.set("true");
        false
    } else {
        error_text.set(String::from("No parse errors..."));
        aria_hint.set("false");
        true
    }
}

#[derive(Props)]
pub struct RecipeComponentProps<'ctx> {
    recipe_id: String,
    sh: StateHandler<'ctx>,
}

#[component]
pub fn Editor<'ctx, G: Html>(cx: Scope<'ctx>, props: RecipeComponentProps<'ctx>) -> View<G> {
    let RecipeComponentProps { recipe_id, sh } = props;
    let store = crate::api::HttpStore::get_from_context(cx);
    let recipe: &Signal<RecipeEntry> =
        create_signal(cx, RecipeEntry::new(&recipe_id, String::new()));
    let text = create_signal(cx, String::from("0"));
    let serving_count_str = create_signal(cx, String::new());
    let serving_count = create_memo(cx, || {
        if let Ok(count) = serving_count_str.get().parse::<i64>() {
            count
        } else {
            0
        }
    });
    let error_text = create_signal(cx, String::from("Parse results..."));
    let aria_hint = create_signal(cx, "false");
    let category = create_signal(cx, "Entree".to_owned());

    spawn_local_scoped(cx, {
        let store = store.clone();
        async move {
            let entry = store
                .fetch_recipe_text(recipe_id.as_str())
                .await
                .expect("Failure getting recipe");
            if let Some(entry) = entry {
                text.set(entry.recipe_text().to_owned());
                if let Some(cat) = entry.category() {
                    category.set(cat.clone());
                }
                recipe.set(entry);
            } else {
                error_text.set("Unable to find recipe".to_owned());
            }
        }
    });

    let id = create_memo(cx, || recipe.get().recipe_id().to_owned());
    let dirty = create_signal(cx, false);
    let ts = create_signal(cx, js_lib::get_ms_timestamp());

    debug!("creating editor view");
    view! {cx,
        div {
            label(for="recipe_category") { "Category" }
            input(name="recipe_category", bind:value=category, on:change=move |_| dirty.set(true))
        }
        div {
            label(for="serving_count") { "Serving Count" }
            input(name="serving_count", bind:value=serving_count_str, on:change=move |_| dirty.set(true))
        }
        div {
            div(class="row-flex") {
                label(for="recipe_text", class="block align-stretch expand-height") { "Recipe: " }
                textarea(class="width-third", name="recipe_text", bind:value=text, aria-invalid=aria_hint.get(), cols="50", rows=20, on:change=move |_| {
                    dirty.set(true);
                    check_recipe_parses(text.get_untracked().as_str(), error_text, aria_hint);
                }, on:input=move |_| {
                    let current_ts = js_lib::get_ms_timestamp();
                    if (current_ts - *ts.get_untracked()) > 100 {
                        check_recipe_parses(text.get_untracked().as_str(), error_text, aria_hint);
                        ts.set(current_ts);
                    }
                })
            }
            div(class="parse") { (error_text.get()) }
        }
        div {
            button(on:click=move |_| {
                let unparsed = text.get_untracked();
                if check_recipe_parses(unparsed.as_str(), error_text, aria_hint) {
                    debug!("triggering a save");
                    if !*dirty.get_untracked() {
                        debug!("Recipe text is unchanged");
                        return;
                    }
                    debug!("Recipe text is changed");
                    let category = category.get_untracked();
                    let category = if category.is_empty() {
                        None
                    } else {
                        Some(category.as_ref().clone())
                    };
                    let recipe_entry = RecipeEntry {
                                    id: id.get_untracked().as_ref().clone(),
                                    text: text.get_untracked().as_ref().clone(),
                                    category,
                                    serving_count: Some(*serving_count.get()),
                    };
                    sh.dispatch(cx, Message::SaveRecipe(recipe_entry, None));
                    dirty.set(false);
                }
                // TODO(jwall): Show error message if trying to save when recipe doesn't parse.
            }) { "Save" } " "
            button(on:click=move |_| {
                sh.dispatch(cx, Message::RemoveRecipe(id.get_untracked().as_ref().to_owned(), Some(Box::new(|| sycamore_router::navigate("/ui/planning/plan")))));
            }) { "delete" } " "
        }
    }
}

#[component]
fn Steps<G: Html>(cx: Scope, steps: Vec<recipes::Step>) -> View<G> {
    let step_fragments = View::new_fragment(steps.iter().enumerate().map(|(idx, step)| {
        let mut step = step.clone();
        let ingredient_fragments = View::new_fragment(step.ingredients.drain(0..).map(|i| {
            view! {cx,
                li {
                    (i.amt) " " (i.name) " " (i.form.as_ref().map(|f| format!("({})", f)).unwrap_or(String::new()))
                }
            }
        }).collect());
        view! {cx,
            div {
                h3 { "Step " (idx + 1) }
                ul(class="ingredients no-list") {
                    (ingredient_fragments)
                }
                div(class="instructions") {
                    (step.instructions)
                }
            }
        }
    }).collect());
    view! {cx,
            h2 { "Instructions: " }
            div(class="recipe_steps") {
                (step_fragments)
            }
    }
}

#[component]
pub fn Viewer<'ctx, G: Html>(cx: Scope<'ctx>, props: RecipeComponentProps<'ctx>) -> View<G> {
    let RecipeComponentProps { recipe_id, sh } = props;
    let view = create_signal(cx, View::empty());
    let recipe_signal = sh.get_selector(cx, move |state| {
        if let Some(recipe) = state.get().recipes.get(&recipe_id) {
            let title = recipe.title.clone();
            let serving_count = recipe.serving_count.clone();
            let desc = recipe.desc.clone().unwrap_or_else(|| String::new());
            let steps = recipe.steps.clone();
            Some((title, serving_count, desc, steps))
        } else {
            None
        }
    });
    if let Some((title, serving_count, desc, steps)) = recipe_signal.get().as_ref().clone() {
        debug!("Viewing recipe.");
        view.set(view! {cx,
            div(class="recipe") {
                h1(class="recipe_title") { (title) }
                 div(class="serving_count") {
                     "Serving Count: " (serving_count.map(|v| format!("{}", v)).unwrap_or_else(|| "Unconfigured".to_owned()))
                 }
                 div(class="recipe_description") {
                     (desc)
                 }
                Steps(steps)
            }
        });
    }
    view! {cx, (view.get().as_ref()) }
}
