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
mod app_state;
mod components;
mod pages;
mod router_integration;
mod service;
mod web;

use router_integration::DeriveRoute;
use sycamore::prelude::*;
use tracing_browser_subscriber;
use wasm_bindgen::prelude::wasm_bindgen;

use web::UI;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    tracing_browser_subscriber::configure_as_global_default();
    let root = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector("#main")
        .unwrap()
        .unwrap();

    sycamore::hydrate_to(|| view! { UI(None) }, &root);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_to_string(path: &str) -> String {
    use app_state::AppRoutes;

    let route = <AppRoutes as DeriveRoute>::from(&(String::new(), path.to_owned(), String::new()));
    sycamore::render_to_string(|| view! { UI(Some(route)) })
}
