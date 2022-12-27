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

use crate::app_state::StateHandler;
use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router};

use crate::pages::*;

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
    #[to("/staples")]
    Staples,
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

#[derive(Props)]
pub struct HandlerProps<'ctx> {
    sh: StateHandler<'ctx>,
}

#[component]
pub fn Handler<'ctx, G: Html>(cx: Scope<'ctx>, props: HandlerProps<'ctx>) -> View<G> {
    let HandlerProps { sh } = props;
    use ManageRoutes::*;
    use PlanningRoutes::*;
    view! {cx,
        Router(
            integration=HistoryIntegration::new(),
            view=move |cx: Scope, route: &ReadSignal<Routes>| {
                match route.get().as_ref() {
                    Routes::Planning(Plan) => view! {cx,
                        PlanPage(sh)
                    },
                    Routes::Planning(Inventory) => view! {cx,
                        InventoryPage(sh)
                    },
                    Routes::Planning(Cook) => view! {cx,
                        CookPage(sh)
                    },
                    Routes::Login => view! {cx,
                        LoginPage(sh)
                    },
                    Routes::Recipe(RecipeRoutes::View(id)) => view! {cx,
                        RecipeViewPage(recipe=id.clone(), sh=sh)
                    },
                    Routes::Recipe(RecipeRoutes::Edit(id)) => view! {cx,
                        RecipeEditPage(recipe=id.clone(), sh=sh)
                    },
                    Routes::Manage(Categories) => view! {cx,
                        CategoryPage(sh)
                    },
                    Routes::Manage(NewRecipe) => view! {cx,
                        AddRecipePage(sh)
                    },
                    Routes::Manage(Staples) => view! {cx,
                        StaplesPage(sh)
                    },
                    Routes::NotFound
                    | Routes::Manage(ManageRoutes::NotFound)
                    | Routes::Planning(PlanningRoutes::NotFound)
                    | Routes::Recipe(RecipeRoutes::NotFound) => view! {cx,
                        // TODO(Create a real one)
                        PlanPage(sh)
                    },
                }

            },
        )
    }
}
