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
use crate::pages::*;
use crate::{
    app_state::*,
    components::*,
    router_integration::*,
    service::{self, AppService},
};
use tracing::{error, info, instrument};

use sycamore::{futures::spawn_local_scoped, prelude::*};

#[instrument]
fn route_switch<G: Html>(cx: Scope, route: &ReadSignal<AppRoutes>) -> View<G> {
    // NOTE(jwall): This needs to not be a dynamic node. The rules around
    // this are somewhat unclear and underdocumented for Sycamore. But basically
    // avoid conditionals in the `view!` macro calls here.
    match route.get().as_ref() {
        AppRoutes::Plan => view! {cx,
            PlanPage()
        },
        AppRoutes::Inventory => view! {cx,
            InventoryPage()
        },
        AppRoutes::Login => view! {cx,
            LoginPage()
        },
        AppRoutes::Cook => view! {cx,
            CookPage()
        },
        AppRoutes::Recipe(idx) => view! {cx,
            RecipePage(recipe=idx.clone())
        },
        AppRoutes::Categories => view! {cx,
            CategoryPage()
        },
        AppRoutes::NotFound => view! {cx,
            // TODO(Create a real one)
            PlanPage()
        },
        AppRoutes::Error(ref e) => {
            let e = e.clone();
            view! {cx,
                "Error: " (e)
            }
        }
    }
}

#[instrument]
#[component]
pub fn UI<G: Html>(cx: Scope) -> View<G> {
    let app_service = AppService::new(service::HttpStore::new("/api/v1".to_owned()));
    provide_context(cx, app_service.clone());
    info!("Starting UI");

    let view = create_signal(cx, View::empty());
    // FIXME(jwall): We need a way to trigger refreshes when required. Turn this
    // into a create_effect with a refresh signal stored as a context.
    spawn_local_scoped(cx, {
        let mut app_service = crate::service::get_appservice_from_context(cx).clone();
        async move {
            if let Err(err) = app_service.synchronize().await {
                error!(?err);
            };
            view.set(view! { cx,
                div(class="app") {
                    Header()
                    Router(RouterProps {
                        route: AppRoutes::Plan,
                        route_select: route_switch,
                        browser_integration: BrowserIntegration::new(),
                    })
                }
            });
        }
    });

    view! { cx, (view.get().as_ref()) }
}
