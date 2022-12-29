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

use crate::app_state::{self, Message, StateHandler};

#[derive(Props)]
pub struct RecipeCheckBoxProps<'ctx> {
    pub i: String,
    pub title: &'ctx ReadSignal<String>,
    pub sh: StateHandler<'ctx>,
}

#[instrument(skip(props, cx), fields(
    id=%props.i,
    title=%props.title.get()
))]
#[component]
pub fn RecipeSelection<'ctx, G: Html>(
    cx: Scope<'ctx>,
    props: RecipeCheckBoxProps<'ctx>,
) -> View<G> {
    let RecipeCheckBoxProps { i, title, sh } = props;
    let state = app_state::State::get_from_context(cx);
    // This is total hack but it works around the borrow issues with
    // the `view!` macro.
    let id = Rc::new(i);
    let count = create_signal(
        cx,
        format!(
            "{}",
            state
                .get_recipe_count_by_index(id.as_ref())
                .unwrap_or_else(|| state.set_recipe_count_by_index(id.as_ref(), 0))
        ),
    );
    create_effect(cx, {
        let id = id.clone();
        let state = app_state::State::get_from_context(cx);
        move || {
            if let Some(usize_count) = state.get_recipe_count_by_index(id.as_ref()) {
                count.set(format!("{}", *usize_count.get()));
            }
        }
    });
    let title = title.get().clone();
    let for_id = id.clone();
    let href = format!("/ui/recipe/view/{}", id);
    let name = format!("recipe_id:{}", id);
    view! {cx,
        div() {
            label(for=for_id) { a(href=href) { (*title) } }
            input(type="number", class="item-count-sel", min="0", bind:value=count, name=name, on:change=move |_| {
                debug!(idx=%id, count=%(*count.get()), "setting recipe count");
                sh.dispatch(cx, Message::UpdateRecipeCount(id.as_ref().clone(), count.get().parse().expect("Count is not a valid usize")));
            })
        }
    }
}
