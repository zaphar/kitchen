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
use crate::components::Recipe;

use sycamore::prelude::*;
use tracing::{debug, instrument};

use crate::service::get_appservice_from_context;

#[instrument]
#[component(RecipeList<G>)]
pub fn recipe_list() -> View<G> {
    let app_service = get_appservice_from_context();
    let menu_list = create_memo(move || app_service.get_menu_list());
    view! {
        h1 { "Recipe List" }
        div() {
            Indexed(IndexedProps{
                iterable: menu_list,
                template: |(idx, _count)| {
                    debug!(idx=%idx, "Rendering recipe");
                    let idx = Signal::new(idx);
                    view ! {
                        Recipe(idx.handle())
                        hr()
                    }
                }
            })
        }
    }
}
