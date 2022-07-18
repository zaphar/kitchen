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

use sycamore::prelude::*;
#[cfg(feature = "web")]
use tracing_wasm;
use wasm_bindgen::prelude::wasm_bindgen;

use web::UI;

#[wasm_bindgen(start)]
pub fn main() {
    if cfg!(feature = "web") {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();
    }
    sycamore::render(|| view! { UI() });
}
