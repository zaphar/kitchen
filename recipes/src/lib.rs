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

use std::collections::BTreeMap;

use unit::*;

/// A Recipe with a title, description, and a series of steps.
pub struct Recipe {
    pub title: String,
    pub desc: String,
    pub steps: Vec<Step>,
}

impl Recipe {
    pub fn new(title: String, desc: String) -> Self {
        Self {
            title,
            desc,
            steps: Vec::new(),
        }
    }

    /// Add steps to the end of the recipe.
    pub fn add_steps(&mut self, steps: Vec<Step>) {
        self.steps.extend(steps.into_iter());
    }

    /// Add a single step to the end of the recipe.
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    /// Get entire ingredients list for each step of the recipe. With duplicate
    /// ingredients added together.
    pub fn get_ingredients(&self) -> BTreeMap<IngredientKey, Ingredient> {
        use Measure::{Count, Gram, Volume};
        self.steps
            .iter()
            .map(|s| s.ingredients.iter())
            .flatten()
            .fold(BTreeMap::new(), |mut acc, i| {
                let key = i.key();
                if !acc.contains_key(&key) {
                    acc.insert(key, i.clone());
                } else {
                    let amt = match (acc[&key].amt, i.amt) {
                        (Volume(rvm), Volume(lvm)) => Volume(lvm + rvm),
                        (Count(lqty), Count(rqty)) => Count(lqty + rqty),
                        (Gram(lqty), Gram(rqty)) => Gram(lqty + rqty),
                        _ => unreachable!(),
                    };
                    acc.get_mut(&key).map(|i| i.amt = amt);
                }
                return acc;
            })
    }
}

/// A Recipe step. It has the time for the step if there is one, instructions, and an ingredients
/// list.
pub struct Step {
    pub prep_time: Option<std::time::Duration>,
    pub instructions: String,
    pub ingredients: Vec<Ingredient>,
}

/// Form of the ingredient.
#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Clone)]
pub enum Form {
    Whole, // default
    Chopped,
    Minced,
    Sliced,
    Ground,
    Mashed,
    Custom(String),
}

/// Unique identifier for an Ingredient. Ingredients are identified by name, form,
/// and measurement type. (Volume, Count, Weight)
#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct IngredientKey(String, Form, String);

/// Ingredient in a recipe. The `name` and `form` fields with the measurement type
/// uniquely identify an ingredient.
#[derive(Clone)]
pub struct Ingredient {
    pub name: String,
    pub form: Form,
    pub amt: Measure,
    pub category: String,
}

impl Ingredient {
    pub fn new<S: Into<String>>(name: S, form: Form, amt: Measure, category: S) -> Self {
        Self {
            name: name.into(),
            form,
            amt,
            category: category.into(),
        }
    }

    /// Unique identifier for this Ingredient.
    pub fn key(&self) -> IngredientKey {
        return IngredientKey(
            self.name.clone(),
            self.form.clone(),
            self.amt.measure_type(),
        );
    }
}

impl std::fmt::Display for Ingredient {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(w, "{} {}", self.amt, self.name)?;
        write!(
            w,
            " ({})",
            match self.form {
                Form::Whole => return Ok(()),
                Form::Chopped => "chopped",
                Form::Minced => "minced",
                Form::Sliced => "sliced",
                Form::Ground => "ground",
                Form::Mashed => "mashed",
                Form::Custom(ref s) => return write!(w, " ({})", s),
            }
        )
    }
}

#[cfg(test)]
mod test;
