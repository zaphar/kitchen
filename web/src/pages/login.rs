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
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use tracing::{debug, info};

use crate::app_state::{Message, StateHandler};

#[component]
pub fn LoginForm<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    let username = create_signal(cx, "".to_owned());
    let password = create_signal(cx, "".to_owned());
    view! {cx,
        form() {
            label(for="username") { "Username" }
            input(type="text", id="username", bind:value=username)
            label(for="password") { "Password" }
            input(type="password", bind:value=password)
            input(type="button", value="Login", on:click=move |_| {
                info!("Attempting login request");
                let (username, password) = ((*username.get_untracked()).clone(), (*password.get_untracked()).clone());
                if username != "" && password != "" {
                    spawn_local_scoped(cx, async move {
                        let store = crate::api::HttpStore::get_from_context(cx);
                        debug!("authenticating against ui");
                        if let Some(user_data) = store.authenticate(username, password).await {
                            sh.dispatch(cx, Message::SetUserData(user_data));
                            sh.dispatch(cx, Message::LoadState(Some(Box::new(|| sycamore_router::navigate("/ui/planning/plan")))));
                        }
                    });
                }
                debug!("triggering login click subscribers");
            }) {  }
        }
    }
}

#[component]
pub fn LoginPage<'ctx, G: Html>(cx: Scope<'ctx>, sh: StateHandler<'ctx>) -> View<G> {
    view! {cx,
            LoginForm(sh)
    }
}
