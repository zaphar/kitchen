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
use js_sys::Date;
use tracing::error;
use wasm_bindgen::JsCast;
use web_sys::{window, Element, Storage, Window};

pub fn get_storage() -> Storage {
    get_window().local_storage()
        .expect("Failed to get storage")
        .expect("No storage available")
}

pub fn get_ms_timestamp() -> u32 {
    Date::new_0().get_milliseconds()
}

pub fn get_window() -> Window {
    window()
        .expect("No window present")
}

pub fn get_element_by_id<E>(id: &str) -> Result<Option<E>, Element>
where
    E: JsCast,
{
    match get_window().document()
        .expect("No document in window")
        .get_element_by_id(id)
    {
        Some(e) => e.dyn_into::<E>().map(|e| Some(e)),
        None => Ok(None),
    }
}

pub trait LogFailures<V, E> {
    fn swallow_and_log(self);
}

impl<E> LogFailures<(), E> for Result<(), E>
where
    E: std::fmt::Debug,
{
    fn swallow_and_log(self) {
        if let Err(e) = self {
            error!(err = ?e, "Error: ");
        }
    }
}
