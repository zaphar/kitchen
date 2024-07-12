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
use sycamore::{futures::spawn_local_scoped, prelude::*};
use tracing::{debug, info, instrument};

use crate::app_state::Message;
use crate::{api, routing::Handler as RouteHandler};

#[instrument]
#[component]
pub fn UI<G: Html>(cx: Scope) -> View<G> {
    let view = create_signal(cx, View::empty());
    api::HttpStore::provide_context(cx, "/api".to_owned());
    let store = api::HttpStore::get_from_context(cx).as_ref().clone();
    info!("Starting UI");
    spawn_local_scoped(cx, {
        async move {
            let local_store = api::LocalStore::new();
            let app_state = if let Some(app_state) = local_store.fetch_app_state().await {
                app_state
            } else {
                crate::app_state::AppState::new()
            };
            debug!(?app_state, "Loaded app state from local storage");
            let sh = crate::app_state::get_state_handler(cx, app_state, store);
            sh.dispatch(cx, Message::LoadState(None));
            view.set(view! { cx,
                RouteHandler(sh=sh)
            });
        }
    });

    view! { cx, (view.get().as_ref()) }
}
