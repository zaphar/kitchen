// Copyright 2022 Jeremy Wall (jeremy@marzhillstudios.com)
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

use crate::components::tabs::*;

mod edit;
mod view;
pub use edit::*;
pub use view::*;

#[derive(Debug, Props)]
pub struct RecipePageProps {
    pub recipe: String,
}

#[derive(Props)]
pub struct PageState<'a, G: Html> {
    pub recipe: String,
    pub children: Children<'a, G>,
    pub selected: Option<String>,
}

#[component]
pub fn RecipePage<'ctx, G: Html>(cx: Scope<'ctx>, state: PageState<'ctx, G>) -> View<G> {
    let PageState {
        children,
        selected,
        recipe,
    } = state;
    let children = children.call(cx);
    let recipe_tabs: Vec<(String, &'static str)> = vec![
        (format!("/ui/recipe/view/{}", recipe), "View"),
        (format!("/ui/recipe/edit/{}", recipe), "Edit"),
    ];
    view! {cx,
        TabbedView(
            selected= selected,
            tablist=recipe_tabs,
        ) { (children) }
    }
}
