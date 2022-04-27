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
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    pub fn debug(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => {
        if cfg!(feature="web") {
            use crate::typings::log;
            log(&format_args!($($t)*).to_string());
        } else if cfg!(feature="ssr") {
            println!($($t)*);
        }
    }
}

#[macro_export]
macro_rules! console_debug {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => {{
        if cfg!(feature="web") {
            use crate::typings::debug;
            debug(&format_args!($($t)*).to_string());
        } else if cfg!(feature="ssr") {
            print!("DEBUG: ");
            println!($($t)*);
        }
    }}
}

#[macro_export]
macro_rules! console_error {
    // Note that this is using the `error` function imported above during
    // `bare_bones`
    ($($t:tt)*) => {{
        if cfg!(feature="web")
        {
            use crate::typings::error;
            error(&format_args!($($t)*).to_string());
        } else if cfg!(feature="ssr") {
            print!("ERROR: ");
            println!($($t)*);
        };
    }}
}

#[macro_export]
macro_rules! console_warn {
    // Note that this is using the `warn` function imported above during
    // `bare_bones`
    ($($t:tt)*) => {{
        if cfg!("web") {
            use crate::typings::warn;
            (warn(&format_args!($($t)*).to_string()))
        } else if cfg!(feature="ssr") {
            print!("WARN: ");
            (println!($($t)*))
        }
    }}
}
