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

use sycamore::{context::use_context, prelude::*};
use tracing::{debug, instrument};

use crate::service::AppService;

pub struct RecipeCheckBoxProps {
    pub i: usize,
    pub title: ReadSignal<String>,
}

#[instrument(skip(props), fields(
    idx=%props.i,
    title=%props.title.get()
))]
#[component(RecipeSelection<G>)]
pub fn recipe_selection(props: RecipeCheckBoxProps) -> View<G> {
    let app_service = use_context::<AppService>();
    // This is total hack but it works around the borrow issues with
    // the `view!` macro.
    let i = props.i;
    let id_as_str = Rc::new(format!("{}", i));
    let id_cloned_2 = id_as_str.clone();
    let count = Signal::new(format!("{}", app_service.get_recipe_count_by_index(i)));
    view! {
        div() {
            label(for=id_cloned_2) { (props.title.get()) }
            input(type="number", class="item-count-sel", min="0", bind:value=count.clone(), name=format!("recipe_id:{}", i), value=id_as_str.clone(), on:change=move |_| {
                let mut app_service = app_service.clone();
                debug!(idx=%i, count=*count.get(), "setting recipe count");
                app_service.set_recipe_count_by_index(i, count.get().parse().unwrap());
            })
        }
    }
}
