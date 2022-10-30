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
use crate::components::tabs::*;
use sycamore::prelude::*;

pub mod categories;

#[derive(Prop)]
pub struct PageState<'a, G: Html> {
    pub children: Children<'a, G>,
    pub selected: Option<String>,
}

#[component]
pub fn ManagePage<'a, G: Html>(cx: Scope<'a>, state: PageState<'a, G>) -> View<G> {
    let PageState { children, selected } = state;
    let children = children.call(cx);
    let manage_tabs: Vec<(&'static str, &'static str)> = vec![("/ui/categories", "Categories")];

    view! {cx,
        TabbedView(
            selected=selected,
            tablist=manage_tabs,
        ) { (children) }
    }
}
