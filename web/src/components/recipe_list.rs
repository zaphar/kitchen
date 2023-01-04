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
use crate::{app_state::StateHandler, components::recipe::Viewer};

use sycamore::prelude::*;
use tracing::{debug, instrument};

#[instrument(skip_all)]
#[component]
pub fn RecipeList<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let menu_list = sh.get_selector(cx, |state| {
        state
            .get()
            .recipe_counts
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .filter(|(_, v)| *(v) != 0)
            .collect()
    });
    view! {cx,
        h1 { "Recipe List" }
        div() {
            Indexed(
                iterable=menu_list,
                view= move |cx, (id, _count)| {
                    debug!(id=%id, "Rendering recipe");
                    view ! {cx,
                        Viewer(recipe_id=id, sh=sh)
                        hr()
                    }
                }
            )
        }
    }
}
