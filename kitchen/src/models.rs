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
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct RecipeRecord {
    pub id: Uuid,
    pub recipe_text: String,
}

impl RecipeRecord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_id_and_text(id: Uuid, recipe_text: String) -> Self {
        Self { id, recipe_text }
    }
}

impl std::default::Default for RecipeRecord {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            recipe_text: String::new(),
        }
    }
}

#[derive(Queryable, Debug)]
pub struct CategoriesRecord {
    pub id: Uuid,
    pub categories_text: String,
}

impl std::default::Default for CategoriesRecord {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            categories_text: String::new(),
        }
    }
}

impl CategoriesRecord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_id_and_text(id: Uuid, categories_text: String) -> Self {
        Self {
            id,
            categories_text,
        }
    }
}
