// Copyright 2021 Jeremy Wall
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
use std::convert::From;

use recipe_store::{RecipeStore, SqliteBackend};
use recipes::*;

pub struct Api {
    store: SqliteBackend,
}

impl Api {
    pub fn new_recipe_from_str(&self, input: &str) {}

    pub fn new_mealplan_from_str(&self, input: &str) {}
}

impl From<SqliteBackend> for Api {
    fn from(store: SqliteBackend) -> Self {
        Api { store }
    }
}
