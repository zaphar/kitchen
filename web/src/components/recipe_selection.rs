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

use crate::app_state::{Message, StateHandler};

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
    let id = Rc::new(i);
    let id_clone = id.clone();
    let count = create_signal(
        cx,
        sh.get_value(
            |state| match state.get_untracked().recipe_counts.get(id_clone.as_ref()) {
                Some(count) => format!("{}", count),
                None => "0".to_owned(),
            },
        ),
    );
    let title = title.get().clone();
    let for_id = id.clone();
    let href = format!("/ui/recipe/view/{}", id);
    let name = format!("recipe_id:{}", id);
    view! {cx,
        div() {
            label(for=for_id) { a(href=href) { (*title) } }
            input(type="number", class="item-count-sel", min="0", value=count, name=name, on:change=move |_| {
                debug!(idx=%id, count=%(*count.get()), "setting recipe count");
                sh.dispatch(cx, Message::UpdateRecipeCount(id.as_ref().clone(), count.get().parse().expect("Count is not a valid usize")));
            })
        }
    }
}
