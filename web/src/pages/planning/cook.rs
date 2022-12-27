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
use sycamore::prelude::*;

use super::PlanningPage;
use crate::{app_state::StateHandler, components::recipe_list::*};

#[component]
pub fn CookPage<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    view! {cx,
        PlanningPage(
            selected=Some("Cook".to_owned()),
        ) { RecipeList() }
    }
}
