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
use web_sys::{window, Storage};

pub fn get_storage() -> Storage {
    window()
        .expect("No Window Present")
        .local_storage()
        .expect("Failed to get storage")
        .expect("No storage available")
}

pub fn get_ms_timestamp() -> u32 {
    Date::new_0().get_milliseconds()
}
