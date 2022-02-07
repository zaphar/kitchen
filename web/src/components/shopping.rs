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
use crate::console_log;
use crate::service::AppService;
use std::rc::Rc;

use sycamore::{context::use_context, prelude::*};

#[component(RecipeSelector<G>)]
pub fn recipe_selector() -> View<G> {
    let app_service = use_context::<AppService>();
    let titles = create_memo(cloned!(app_service => move || {
        app_service.get_recipes().get().iter().map(|(i, r)| (*i, r.title.clone())).collect::<Vec<(usize, String)>>()
    }));
    view! {
        fieldset(class="recipe_selector") {
            Keyed(KeyedProps{
                iterable: titles,
                template: |(i, title)| {
                    // This is total hack but it works around the borrow issues with
                    // the `view!` macro.
                    let id_as_str = Rc::new(format!("{}", i));
                    let id_cloned = id_as_str.clone();
                    let id_cloned_2 = id_as_str.clone();
                    view! {
                        input(type="checkbox", name="recipe_id", value=id_as_str.clone(), on:click=move |_| {
                            console_log!("clicked checkbox for id {}", id_cloned);
                        })
                        label(for=id_cloned_2) { (title) } }
                },
                key: |(i, title)| (*i, title.clone()),
            })
        }
    }
}

#[component(MenuView<G>)]
pub fn shopping_view() -> View<G> {
    view! {
        h1 {
            "Select your recipes"
        }
        RecipeSelector()
    }
}
