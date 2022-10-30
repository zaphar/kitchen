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
use base64;
use reqwasm::http;
use sycamore::{futures::spawn_local_scoped, prelude::*};
use tracing::{debug, error, info};

fn token68(user: String, pass: String) -> String {
    base64::encode(format!("{}:{}", user, pass))
}

async fn authenticate(user: String, pass: String) -> bool {
    debug!(
        username = user,
        password = pass,
        "attempting login request against api."
    );
    let result = http::Request::get("/api/v1/auth")
        .header(
            "Authorization",
            format!("Basic {}", token68(user, pass)).as_str(),
        )
        .send()
        .await;
    if let Ok(resp) = &result {
        if resp.status() == 200 {
            return true;
        }
        error!(status = resp.status(), "Login was unsuccessful")
    } else {
        error!(err=?result.unwrap_err(), "Failed to send auth request");
    }
    return false;
}

#[component]
pub fn LoginForm<G: Html>(cx: Scope) -> View<G> {
    let username = create_signal(cx, "".to_owned());
    let password = create_signal(cx, "".to_owned());
    let clicked = create_signal(cx, ("".to_owned(), "".to_owned()));
    create_effect(cx, move || {
        let (username, password) = (*clicked.get()).clone();
        if username != "" && password != "" {
            spawn_local_scoped(cx, async move {
                debug!("authenticating against ui");
                // TODO(jwall): Navigate to plan if the below is successful.
                authenticate(username, password).await;
            });
        }
    });
    view! {cx,
        form() {
            label(for="username") { "Username" }
            input(type="text", id="username", bind:value=username)
            label(for="password") { "Password" }
            input(type="password", bind:value=password)
            input(type="button", value="Login", on:click=move |_| {
                info!("Attempting login request");
                clicked.set(((*username.get_untracked()).clone(), (*password.get_untracked()).clone()));
                debug!("triggering login click subscribers");
                clicked.trigger_subscribers();
            }) {  }
        }
    }
}

#[component]
pub fn LoginPage<G: Html>(cx: Scope) -> View<G> {
    view! {cx,
            LoginForm()
    }
}
