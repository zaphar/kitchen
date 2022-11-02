// Copyright 2022 zaphar
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
//use sycamore_router::{HistoryIntegration, Route, Router};
use sycamore_router::{HistoryIntegration, Route, Router};
use tracing::instrument;

use crate::pages::*;

//mod router;
//use router::{HistoryIntegration, Router};

#[instrument]
fn route_switch<'a, G: Html>(cx: Scope<'a>, route: &'a ReadSignal<Routes>) -> View<G> {
    // NOTE(jwall): This needs to not be a dynamic node. The rules around
    // this are somewhat unclear and underdocumented for Sycamore. But basically
    // avoid conditionals in the `view!` macro calls here.
    view! {cx,
        (match route.get().as_ref() {
            Routes::Plan => view! {cx,
                PlanPage()
            },
            Routes::Inventory => view! {cx,
                InventoryPage()
            },
            Routes::Login => view! {cx,
                LoginPage()
            },
            Routes::Cook => view! {cx,
                CookPage()
            },
            Routes::Recipe(idx) => view! {cx,
                RecipePage(recipe=idx.clone())
            },
            Routes::Categories => view! {cx,
                CategoryPage()
            },
            Routes::NewRecipe => view! {cx,
                AddRecipePage()
            },
            Routes::NotFound => view! {cx,
                // TODO(Create a real one)
                PlanPage()
            },
        })
    }
}

#[derive(Route, Debug)]
pub enum Routes {
    #[to("/ui/plan")]
    Plan,
    #[to("/ui/inventory")]
    Inventory,
    #[to("/ui/cook")]
    Cook,
    #[to("/ui/recipe/<id>")]
    Recipe(String),
    #[to("/ui/add_recipe")]
    NewRecipe,
    #[to("/ui/categories")]
    Categories,
    #[to("/ui/login")]
    Login,
    #[not_found]
    NotFound,
}

#[component]
pub fn Handler<G: Html>(cx: Scope) -> View<G> {
    view! {cx,
        Router(
            integration=HistoryIntegration::new(),
            view=route_switch,
        )
    }
}
