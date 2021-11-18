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
pub mod parse;
pub mod unit;

use std::collections::BTreeMap;

use chrono::NaiveDate;
use uuid::{self, Uuid};

use unit::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Mealplan {
    pub id: uuid::Uuid,
    pub start_date: Option<NaiveDate>,
    pub recipes: Vec<Recipe>,
}

impl Mealplan {
    pub fn new() -> Self {
        Self::new_id(uuid::Uuid::new_v4())
    }

    pub fn new_id(id: Uuid) -> Self {
        Self {
            id: id,
            start_date: None,
            recipes: Vec::new(),
        }
    }

    pub fn with_start_date(mut self, start_date: NaiveDate) -> Self {
        self.start_date = Some(start_date);
        self
    }

    pub fn add_recipes<Iter>(&mut self, recipes: Iter)
    where
        Iter: IntoIterator<Item = Recipe>,
    {
        self.recipes.extend(recipes.into_iter())
    }
}

/// A Recipe with a title, description, and a series of steps.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    pub id: uuid::Uuid,
    pub title: String,
    pub desc: Option<String>,
    pub steps: Vec<Step>,
}

impl Recipe {
    pub fn new<S: Into<String>>(title: S, desc: Option<S>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            title: title.into(),
            desc: desc.map(|s| s.into()),
            steps: Vec::new(),
        }
    }

    pub fn new_with_id<S: Into<String>>(id: uuid::Uuid, title: S, desc: Option<S>) -> Self {
        Self {
            id: id,
            title: title.into(),
            desc: desc.map(|s| s.into()),
            steps: Vec::new(),
        }
    }

    pub fn with_steps<Iter>(mut self, steps: Iter) -> Self
    where
        Iter: IntoIterator<Item = Step>,
    {
        self.add_steps(steps);
        self
    }

    /// Add steps to the end of the recipe.
    pub fn add_steps<Iter>(&mut self, steps: Iter)
    where
        Iter: IntoIterator<Item = Step>,
    {
        self.steps.extend(steps.into_iter());
    }

    /// Add a single step to the end of the recipe.
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    /// Get entire ingredients list for each step of the recipe. With duplicate
    /// ingredients added together.
    pub fn get_ingredients(&self) -> BTreeMap<IngredientKey, Ingredient> {
        use Measure::{Count, Volume, Weight};
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
                        (Weight(lqty), Weight(rqty)) => Weight(lqty + rqty),
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
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub prep_time: Option<std::time::Duration>,
    pub instructions: String,
    pub ingredients: Vec<Ingredient>,
}

impl Step {
    pub fn new<S: Into<String>>(prep_time: Option<std::time::Duration>, instructions: S) -> Self {
        Self {
            prep_time: prep_time,
            instructions: instructions.into(),
            ingredients: Vec::new(),
        }
    }

    pub fn with_ingredients<Iter>(mut self, ingredients: Iter) -> Step
    where
        Iter: IntoIterator<Item = Ingredient>,
    {
        self.add_ingredients(ingredients);
        self
    }

    pub fn add_ingredients<Iter>(&mut self, ingredients: Iter)
    where
        Iter: IntoIterator<Item = Ingredient>,
    {
        self.ingredients.extend(ingredients.into_iter());
    }

    pub fn add_ingredient(&mut self, ingredient: Ingredient) {
        self.ingredients.push(ingredient);
    }
}

/// Unique identifier for an Ingredient. Ingredients are identified by name, form,
/// and measurement type. (Volume, Count, Weight)
#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct IngredientKey(String, Option<String>, String);

/// Ingredient in a recipe. The `name` and `form` fields with the measurement type
/// uniquely identify an ingredient.
#[derive(Debug, Clone, PartialEq)]
pub struct Ingredient {
    pub id: Option<i64>, // TODO(jwall): use uuid instead?
    pub name: String,
    pub form: Option<String>,
    pub amt: Measure,
    pub category: String,
}

impl Ingredient {
    pub fn new<S: Into<String>>(name: S, form: Option<String>, amt: Measure, category: S) -> Self {
        Self {
            id: None,
            name: name.into(),
            form,
            amt,
            category: category.into(),
        }
    }

    pub fn new_with_id<S: Into<String>>(
        id: i64,
        name: S,
        form: Option<String>,
        amt: Measure,
        category: S,
    ) -> Self {
        Self {
            id: Some(id),
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
        if let Some(f) = &self.form {
            write!(w, " ({})", f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;
