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

#[derive(Clone)]
pub struct TabState<G: GenericNode> {
    pub inner: View<G>,
}

#[component(TabbedView<G>)]
pub fn tabbed_view(state: TabState<G>) -> View<G> {
    cloned!((state) => view! {
        header(class="no-print") {
            nav {
                ul {
                    li { a(href="/ui/plan", class="no-print") { "Plan" } " > "
                    }
                    li { a(href="/ui/inventory", class="no-print") { "Inventory" } " > "
                    }
                    li { a(href="/ui/cook", class="no-print") { "Cook" }
                    } " | "
                    li { a(href="/ui/categories", class="no-print") { "Categories" }
                    }
                }
                ul {
                    li { a(href="/ui/login") { "Login" } " | " }
                    li { a(href="https://github.com/zaphar/kitchen") { "Github" } }
                }
            }
        }
        main(class=".conatiner-fluid") {
            (state.inner)
        }
    })
}
