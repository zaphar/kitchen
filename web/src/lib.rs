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
mod api;
mod app_state;
mod components;
mod js_lib;
mod pages;
mod routing;
mod web;

use sycamore::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

use web::UI;

fn configure_tracing() {
    console_error_panic_hook::set_once();
    use tracing_subscriber::fmt::format::Pretty;
    use tracing_subscriber::prelude::*;
    use tracing_web::{performance_layer, MakeConsoleWriter};
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .without_time()
        //.with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

#[wasm_bindgen(start)]
pub fn main() {
    configure_tracing();
    sycamore::render(|cx| view! { cx, UI() });
}
