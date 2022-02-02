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
use crate::components::*;
use crate::service::AppService;

use sycamore::{context::use_context, prelude::*};

#[component(Start<G>)]
pub fn start() -> View<G> {
    view! {
        div { "hello chefs!" }
        RecipeList()
    }
}

#[component(RecipeView<G>)]
pub fn recipe_view(idx: usize) -> View<G> {
    let idx = Signal::new(idx);
    view! {
        div { "hello chefs!"}
        RecipeList()
        Recipe(idx.handle())
    }
}

/// Component to list available recipes.
#[component(RecipeList<G>)]
pub fn recipe_list() -> View<G> {
    let app_service = use_context::<AppService>();

    let titles = create_memo(cloned!(app_service => move || {
        app_service.get_recipes().get().iter().map(|(i, r)| (*i, r.title.clone())).collect::<Vec<(usize, String)>>()
    }));
    view! {
        ul(class="recipe_list") {
            Keyed(KeyedProps{
                iterable: titles,
                template: |(i, title)| {
                    view! { li { a(href=format!("/ui/recipe/{}", i)) { (title) } } }
                },
                key: |(i, title)| (*i, title.clone()),
            })
        }
    }
}
