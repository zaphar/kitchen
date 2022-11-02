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
use std::rc::Rc;

use sycamore::prelude::*;
use tracing::{debug, instrument};

use crate::app_state;

#[derive(Props)]
pub struct RecipeCheckBoxProps<'ctx> {
    pub i: String,
    pub title: &'ctx ReadSignal<String>,
}

#[instrument(skip(props, cx), fields(
    idx=%props.i,
    title=%props.title.get()
))]
#[component]
pub fn RecipeSelection<G: Html>(cx: Scope, props: RecipeCheckBoxProps) -> View<G> {
    let state = app_state::State::get_from_context(cx);
    // This is total hack but it works around the borrow issues with
    // the `view!` macro.
    let id = Rc::new(props.i);
    let count = create_signal(
        cx,
        format!(
            "{}",
            state
                .get_recipe_count_by_index(id.as_ref())
                .unwrap_or_else(|| state.set_recipe_count_by_index(id.as_ref(), 0))
        ),
    );
    let title = props.title.get().clone();
    let for_id = id.clone();
    let href = format!("/ui/recipe/{}", id);
    let name = format!("recipe_id:{}", id);
    view! {cx,
        div() {
            label(for=for_id) { a(href=href) { (*title) } }
            input(type="number", class="item-count-sel", min="0", bind:value=count, name=name, on:change=move |_| {
                debug!(idx=%id, count=%(*count.get()), "setting recipe count");
                state.set_recipe_count_by_index(id.as_ref(), count.get().parse().expect("recipe count isn't a valid usize number"));
            })
        }
    }
}
