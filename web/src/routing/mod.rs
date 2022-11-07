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
use sycamore_router::{HistoryIntegration, Route, Router};
use tracing::{debug, instrument};

use crate::pages::*;

#[instrument]
fn route_switch<'a, G: Html>(cx: Scope<'a>, route: &'a ReadSignal<Routes>) -> View<G> {
    // NOTE(jwall): This needs to not be a dynamic node. The rules around
    // this are somewhat unclear and underdocumented for Sycamore. But basically
    // avoid conditionals in the `view!` macro calls here.

    let switcher = |cx: Scope, route: &Routes| {
        debug!(?route, "Dispatching for route");
        match route {
            Routes::Planning(Plan) => view! {cx,
                PlanPage()
            },
            Routes::Planning(Inventory) => view! {cx,
                InventoryPage()
            },
            Routes::Planning(Cook) => view! {cx,
                CookPage()
            },
            Routes::Login => view! {cx,
                LoginPage()
            },
            Routes::Recipe(RecipeRoutes::View(id)) => view! {cx,
                RecipeViewPage(recipe=id.clone())
            },
            Routes::Recipe(RecipeRoutes::Edit(id)) => view! {cx,
                RecipeEditPage(recipe=id.clone())
            },
            Routes::Manage(ManageRoutes::Categories) => view! {cx,
                CategoryPage()
            },
            Routes::Manage(ManageRoutes::NewRecipe) => view! {cx,
                AddRecipePage()
            },
            Routes::NotFound
            | Routes::Manage(ManageRoutes::NotFound)
            | Routes::Planning(PlanningRoutes::NotFound)
            | Routes::Recipe(RecipeRoutes::NotFound) => view! {cx,
                // TODO(Create a real one)
                PlanPage()
            },
        }
    };
    use PlanningRoutes::*;
    view! {cx,
        (switcher(cx, route.get().as_ref()))
    }
}

#[derive(Route, Debug)]
pub enum Routes {
    #[to("/ui/planning/<_..>")]
    Planning(PlanningRoutes),
    #[to("/ui/recipe/<_..>")]
    Recipe(RecipeRoutes),
    #[to("/ui/manage/<_..>")]
    Manage(ManageRoutes),
    #[to("/ui/login")]
    Login,
    #[not_found]
    NotFound,
}

#[derive(Route, Debug)]
pub enum RecipeRoutes {
    #[to("/edit/<id>")]
    Edit(String),
    #[to("/view/<id>")]
    View(String),
    #[not_found]
    NotFound,
}

#[derive(Route, Debug)]
pub enum ManageRoutes {
    #[to("/new_recipe")]
    NewRecipe,
    #[to("/categories")]
    Categories,
    #[not_found]
    NotFound,
}

#[derive(Route, Debug)]
pub enum PlanningRoutes {
    #[to("/plan")]
    Plan,
    #[to("/inventory")]
    Inventory,
    #[to("/cook")]
    Cook,
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
