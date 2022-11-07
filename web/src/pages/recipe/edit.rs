// Copyright 2022 Jeremy Wall (Jeremy@marzhilsltudios.com)
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
use crate::components::recipe::Editor;

use sycamore::prelude::*;
use tracing::instrument;

use super::{RecipePage, RecipePageProps};

#[instrument]
#[component()]
pub fn RecipeEditPage<G: Html>(cx: Scope, props: RecipePageProps) -> View<G> {
    view! {cx,
        RecipePage(
            selected=Some("Edit".to_owned()),
            recipe=props.recipe.clone(),
        ) { Editor(props.recipe) }
    }
}
