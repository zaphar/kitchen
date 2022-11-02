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

use recipes::RecipeEntry;

const STARTER_RECIPE: &'static str = "title: Title Here

Description here.

step:

1 ingredient

Instructions here
";

#[component]
pub fn AddRecipePage<G: Html>(cx: Scope) -> View<G> {
    let entry = create_signal(cx, RecipeEntry(String::new(), String::from(STARTER_RECIPE)));
    let recipe_id = create_signal(cx, String::new());
    let create_recipe_signal = create_signal(cx, ());
    let dirty = create_signal(cx, false);

    create_effect(cx, || {
        let mut entry_for_edit = entry.get_untracked().as_ref().clone();
        // TODO(jwall): This can probably be done more efficiently.
        let id = recipe_id
            .get()
            .as_ref()
            .replace(" ", "_")
            .replace("\n", "")
            .replace("\r", "");
        entry_for_edit.set_recipe_id(id);
        entry.set(entry_for_edit);
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
                // TODO(jwall): Ensure that this id doesn't already exist.
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
        label(for="recipe_id") { "Recipe Id" }
        input(bind:value=recipe_id, type="text", name="recipe_id", id="recipe_id", on:change=move |_| {
            dirty.set(true);
        })
        button(on:click=move |_| {
            create_recipe_signal.trigger_subscribers();
        }) { "Create" }
    }
}
