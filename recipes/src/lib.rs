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

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use unit::*;
use Measure::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Mealplan {
    pub start_date: Option<NaiveDate>,
    pub recipes: Vec<Recipe>,
}

impl Mealplan {
    pub fn new() -> Self {
        Self {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecipeEntry {
    pub id: String,
    pub text: String,
    pub category: Option<String>,
    pub serving_count: Option<i64>,
}

impl RecipeEntry {
    pub fn new<IS: Into<String>, TS: Into<String>>(recipe_id: IS, text: TS) -> Self {
        Self {
            id: recipe_id.into(),
            text: text.into(),
            category: None,
            serving_count: None,
        }
    }

    pub fn set_recipe_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    pub fn recipe_id(&self) -> &str {
        self.id.as_str()
    }

    pub fn set_recipe_text<S: Into<String>>(&mut self, text: S) {
        self.text = text.into();
    }

    pub fn recipe_text(&self) -> &str {
        self.text.as_str()
    }

    pub fn set_category<S: Into<String>>(&mut self, cat: S) {
        self.category = Some(cat.into());
    }

    pub fn category(&self) -> Option<&String> {
        self.category.as_ref()
    }

    pub fn serving_count(&self) -> Option<i64> {
        self.serving_count.clone()
    }
}

/// A Recipe with a title, description, and a series of steps.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Recipe {
    pub title: String,
    pub desc: Option<String>,
    pub steps: Vec<Step>,
}

impl Recipe {
    pub fn new<S: Into<String>>(title: S, desc: Option<S>) -> Self {
        Self {
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
        let mut acc = IngredientAccumulator::new();
        acc.accumulate_from(&self);
        acc.ingredients()
            .into_iter()
            .map(|(k, v)| (k, v.0))
            .collect()
    }
}

pub struct IngredientAccumulator {
    inner: BTreeMap<IngredientKey, (Ingredient, BTreeSet<String>)>,
}

impl IngredientAccumulator {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn accumulate_ingredients_for<'a, Iter, S>(&'a mut self, recipe_title: S, ingredients: Iter)
    where
        Iter: Iterator<Item = &'a Ingredient>,
        S: Into<String>,
    {
        let recipe_title = recipe_title.into();
        for i in ingredients {
            let key = i.key();
            if !self.inner.contains_key(&key) {
                let mut set = BTreeSet::new();
                set.insert(recipe_title.clone());
                self.inner.insert(key, (i.clone(), set));
            } else {
                let amts = match (&self.inner[&key].0.amt, &i.amt) {
                    (Volume(rvm), Volume(lvm)) => vec![Volume(lvm + rvm)],
                    (Count(lqty), Count(rqty)) => vec![Count(lqty + rqty)],
                    (Weight(lqty), Weight(rqty)) => vec![Weight(lqty + rqty)],
                    (Package(lnm, lqty), Package(rnm, rqty)) => {
                        if lnm == rnm {
                            vec![Package(lnm.clone(), lqty + rqty)]
                        } else {
                            vec![
                                Package(lnm.clone(), lqty.clone()),
                                Package(rnm.clone(), rqty.clone()),
                            ]
                        }
                    }
                    _ => unreachable!(),
                };
                for amt in amts {
                    self.inner.get_mut(&key).map(|(i, set)| {
                        i.amt = amt;
                        set.insert(recipe_title.clone());
                    });
                }
            }
        }
    }

    pub fn accumulate_from(&mut self, r: &Recipe) {
        self.accumulate_ingredients_for(
            &r.title,
            r.steps.iter().map(|s| s.ingredients.iter()).flatten(),
        );
    }

    pub fn ingredients(self) -> BTreeMap<IngredientKey, (Ingredient, BTreeSet<String>)> {
        self.inner
    }
}

/// A Recipe step. It has the time for the step if there is one, instructions, and an ingredients
/// list.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Step {
    pub prep_time: Option<std::time::Duration>,
    pub instructions: String,
    pub ingredients: Vec<Ingredient>,
}

impl Step {
    pub fn new<S: Into<String>>(prep_time: Option<std::time::Duration>, instructions: S) -> Self {
        Self {
            prep_time,
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
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Hash, Debug, Deserialize, Serialize)]
pub struct IngredientKey(String, Option<String>, String);

impl IngredientKey {
    pub fn new(name: String, form: Option<String>, measure_type: String) -> Self {
        Self(name, form, measure_type)
    }

    pub fn name(&self) -> &String {
        &self.0
    }

    pub fn form(&self) -> String {
        self.1.clone().unwrap_or_else(|| String::new())
    }

    pub fn measure_type(&self) -> &String {
        &self.2
    }
}

/// Ingredient in a recipe. The `name` and `form` fields with the measurement type
/// uniquely identify an ingredient.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Ingredient {
    pub id: Option<i64>, // TODO(jwall): use uuid instead?
    pub name: String,
    pub form: Option<String>,
    pub amt: Measure,
}

impl Ingredient {
    pub fn new<S: Into<String>>(name: S, form: Option<String>, amt: Measure) -> Self {
        Self {
            id: None,
            name: name.into(),
            form,
            amt,
        }
    }

    pub fn new_with_id<S: Into<String>>(
        id: i64,
        name: S,
        form: Option<String>,
        amt: Measure,
    ) -> Self {
        Self {
            id: Some(id),
            name: name.into(),
            form,
            amt,
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
