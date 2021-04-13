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
pub mod unit;

use unit::*;

pub struct Recipe {
    pub title: String,
    pub desc: String,
    pub steps: Vec<Step>,
}

pub struct Step {
    pub prep_time: std::time::Duration,
    pub instructions: String,
    pub ingredients: Vec<Ingredient>,
}

pub enum Form {
    Whole, // default
    Chopped,
    Minced,
    Sliced,
    Ground,
    Custom(String),
}

pub struct Ingredient {
    pub name: String,
    pub amt: Measure,
    pub form: Form,
    pub category: String,
}

#[cfg(test)]
mod test;
