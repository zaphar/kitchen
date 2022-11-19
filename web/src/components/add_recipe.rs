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
use sycamore::{futures::spawn_local_scoped, prelude::*};
use tracing::{error, info};

use recipes::RecipeEntry;

const STARTER_RECIPE: &'static str = "title: TITLE_PLACEHOLDER

Description here.

step:

1 ingredient

Instructions here
";

#[component]
pub fn AddRecipe<G: Html>(cx: Scope) -> View<G> {
    let recipe_title = create_signal(cx, String::new());
    let create_recipe_signal = create_signal(cx, ());
    let dirty = create_signal(cx, false);

    let entry = create_memo(cx, || {
        RecipeEntry(
            recipe_title
                .get()
                .as_ref()
                .to_lowercase()
                .replace(" ", "_")
                .replace("\n", ""),
            STARTER_RECIPE
                .replace("TITLE_PLACEHOLDER", recipe_title.get().as_str())
                .replace("\r", ""),
        )
    });

    create_effect(cx, move || {
        create_recipe_signal.track();
        if !*dirty.get_untracked() {
            return;
        }
        spawn_local_scoped(cx, {
            let store = crate::api::HttpStore::get_from_context(cx);
            async move {
                let entry = entry.get_untracked();
                // TODO(jwall): Better error reporting here.
                match store.get_recipe_text(entry.recipe_id()).await {
                    Ok(Some(_)) => {
                        // TODO(jwall): We should tell the user that this id already exists
                        info!(recipe_id = entry.recipe_id(), "Recipe already exists");
                        return;
                    }
                    Ok(None) => {
                        // noop
                    }
                    Err(err) => {
                        // TODO(jwall): We should tell the user that this is failing
                        error!(?err)
                    }
                }
                store
                    .save_recipes(vec![entry.as_ref().clone()])
                    .await
                    .expect("Unable to save New Recipe");
                crate::js_lib::navigate_to_path(&format!("/ui/recipe/{}", entry.recipe_id()))
                    .expect("Unable to navigate to recipe");
            }
        });
    });
    view! {cx,
        label(for="recipe_title") { "Recipe Title" }
        input(bind:value=recipe_title, type="text", name="recipe_title", id="recipe_title", on:change=move |_| {
            dirty.set(true);
        })
        button(on:click=move |_| {
            create_recipe_signal.trigger_subscribers();
        }) { "Create" }
    }
}