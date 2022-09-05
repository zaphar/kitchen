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

use crate::service::get_appservice_from_context;

pub struct RecipeCheckBoxProps {
    pub i: String,
    pub title: ReadSignal<String>,
}

#[instrument(skip(props), fields(
    idx=%props.i,
    title=%props.title.get()
))]
#[component(RecipeSelection<G>)]
pub fn recipe_selection(props: RecipeCheckBoxProps) -> View<G> {
    let app_service = get_appservice_from_context();
    // This is total hack but it works around the borrow issues with
    // the `view!` macro.
    let id = Rc::new(props.i);
    let count = Signal::new(format!(
        "{}",
        app_service.get_recipe_count_by_index(id.as_ref())
    ));
    let for_id = id.clone();
    let href = format!("/ui/recipe/{}", id);
    let name = format!("recipe_id:{}", id);
    let value = id.clone();
    view! {
        div() {
            label(for=for_id) { a(href=href) { (props.title.get()) } }
            input(type="number", class="item-count-sel", min="0", bind:value=count.clone(), name=name, value=value, on:change=cloned!((id) => move |_| {
                let mut app_service = app_service.clone();
                debug!(idx=%id, count=%(*count.get()), "setting recipe count");
                app_service.set_recipe_count_by_index(id.as_ref().to_owned(), count.get().parse().unwrap());
            }))
        }
    }
}
