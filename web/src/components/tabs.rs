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
use tracing::debug;

#[derive(Props)]
pub struct TabState<'a, G: Html> {
    pub children: Children<'a, G>,
    pub selected: Option<String>,
    tablist: Vec<(String, &'static str)>,
}

#[component]
pub fn TabbedView<'a, G: Html>(cx: Scope<'a>, state: TabState<'a, G>) -> View<G> {
    let TabState {
        children,
        selected,
        tablist,
    } = state;
    let children = children.call(cx);
    let menu = View::new_fragment(
        tablist
            .iter()
            .map(|&(ref href, show)| {
                let href = href.clone();
                debug!(?selected, show, "identifying tab");
                let class = if selected.as_ref().map_or(false, |selected| selected == show) {
                    "no-print selected"
                } else {
                    "no-print"
                };
                view! {cx,
                    li(class=class) { a(href=href) { (show) } }
                }
            })
            .collect(),
    );
    view! {cx,
        nav {
            ul(class="tabs") {
                (menu)
            }
        }
        main(class=".conatiner-fluid") {
            (children)
        }
    }
}
