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
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{window, Element, Storage};

pub fn navigate_to_path(path: &str) -> Result<(), JsValue> {
    window()
        .expect("No window present")
        .location()
        .set_pathname(path)
}

pub fn get_element_by_id<E>(id: &str) -> Result<Option<E>, Element>
where
    E: JsCast,
{
    match window()
        .expect("No window present")
        .document()
        .expect("No document in window")
        .get_element_by_id(id)
    {
        Some(e) => e.dyn_into::<E>().map(|e| Some(e)),
        None => Ok(None),
    }
}

pub fn get_storage() -> Storage {
    window()
        .expect("No Window Present")
        .local_storage()
        .expect("Failed to get storage")
        .expect("No storage available")
}

pub fn get_storage_keys() -> Vec<String> {
    let storage = get_storage();
    let mut keys = Vec::new();
    for idx in 0..storage.length().unwrap() {
        keys.push(get_storage().key(idx).unwrap().unwrap())
    }
    keys
}
