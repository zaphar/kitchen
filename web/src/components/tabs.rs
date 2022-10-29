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

use super::Header;
#[derive(Clone, Prop)]
pub struct TabState<G: GenericNode> {
    pub inner: View<G>,
    pub selected: Option<String>,
}

#[component]
pub fn TabbedView<G: Html>(cx: Scope, state: TabState<G>) -> View<G> {
    let tablist = create_signal(
        cx,
        vec![
            ("/ui/plan", "Plan"),
            ("/ui/inventory", "Inventory"),
            ("/ui/cook", "Cook"),
            ("/ui/categories", "Categories"),
        ],
    );
    let TabState { inner, selected } = state;
    view! {cx,
        Header { }
        nav {
            ul(class="tabs") {
                Indexed(
                    iterable=tablist,
                    view=move |cx, (href, show)| {
                        debug!(?selected, show, "identifying tab");
                        let class = if selected.as_ref().map_or(false, |selected| selected == show) {
                            "no-print selected"
                        } else {
                            "no-print"
                        };
                        view! {cx,
                            li(class=class) { a(href=href) { (show) } }
                        }
                    }
                )
            }
            ul {
                li { a(href="/ui/login") { "Login" } " | " }
                li { a(href="https://github.com/zaphar/kitchen") { "Github" } }
            }
        }
        main(class=".conatiner-fluid") {
            (inner)
        }
    }
}
