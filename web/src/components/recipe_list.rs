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
use crate::{app_state, components::Recipe};

use sycamore::prelude::*;
use tracing::{debug, instrument};

#[instrument]
#[component]
pub fn RecipeList<G: Html>(cx: Scope) -> View<G> {
    let state = app_state::State::get_from_context(cx);
    let menu_list = create_memo(cx, move || state.get_menu_list());
    view! {cx,
        h1 { "Recipe List" }
        div() {
            Indexed(
                iterable=menu_list,
                view= |cx, (idx, _count)| {
                    debug!(idx=%idx, "Rendering recipe");
                    view ! {cx,
                        Recipe(idx)
                        hr()
                    }
                }
            )
        }
    }
}
