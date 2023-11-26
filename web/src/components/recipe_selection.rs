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
use crate::components::NumberField;

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
    let id_for_count = id.clone();
    // NOTE(jwall): The below get's a little tricky. We need a separate signal to bind for the
    // this recipes count. But we also want it to automatically update if the app_state
    // recipe count updates. We need to avoid signal update cycles so we have to do this
    // in two steps. We have a read signal that represents changes in the value of the
    // app_states count. We have a Signal that represents the value of this components count.
    // If the app_states count changes and is also different from the components count then we
    // and only then do we set the components count to the app states count.
    let current_count = sh.get_selector(cx, move |state| {
        *state
            .get()
            .recipe_counts
            .get(id_for_count.as_ref())
            .unwrap_or(&0)
    });
    let count = create_signal(cx, *current_count.get_untracked() as f64);
    create_effect(cx, || {
        let updated_count = *current_count.get() as f64;
        if updated_count != *count.get_untracked() {
            count.set(updated_count);
        }
    });

    let title = title.get().clone();
    let href = format!("/ui/recipe/view/{}", id);
    let name = format!("recipe_id:{}", id);
    let for_id = name.clone();
    view! {cx,
        label(for=for_id) { a(href=href) { (*title) } }
        NumberField(name=name, counter=count, min=0.0, on_change=Some(move |_| {
            debug!(idx=%id, count=%(*count.get_untracked()), "setting recipe count");
            sh.dispatch(cx, Message::UpdateRecipeCount(id.as_ref().clone(), *count.get_untracked() as usize));
        }))
    }
}
