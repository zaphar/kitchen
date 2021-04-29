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
use crate::*;
use std::convert::Into;

macro_rules! init_sqlite_store {
    () => {{
        let in_memory =
            SqliteBackend::new_in_memory().expect("We expect in memory connections to succeed");
        in_memory
            .create_schema()
            .expect("We expect the schema creation to succeed");
        let version = in_memory
            .get_schema_version()
            .expect("We expect the version fetch to succeed");
        assert!(version.is_some());
        assert_eq!(version.unwrap(), 0);
        in_memory
    }};
}

#[test]
fn test_schema_creation() {
    let in_memory = init_sqlite_store!();

    in_memory
        .create_schema()
        .expect("create_schema is idempotent");
    let version = in_memory
        .get_schema_version()
        .expect("We expect the version fetch to succeed");
    assert!(version.is_some());
    assert_eq!(version.unwrap(), 0);
}

#[test]
fn test_recipe_store_update_roundtrip_full() {
    let mut in_memory = init_sqlite_store!();

    let mut recipe = Recipe::new("my recipe", "my description");
    let mut step1 = Step::new(
        Some(std::time::Duration::from_secs(60 * 30)),
        "mix thoroughly",
    );
    step1.add_ingredients(vec![
        Ingredient::new("flour", None, Measure::cup(1.into()), "dry goods"),
        Ingredient::new(
            "salt",
            Some("Ground".to_owned()),
            Measure::tsp(1.into()),
            "seasoning",
        ),
    ]);
    let step2 = Step::new(None, "combine ingredients");
    recipe.add_steps(vec![step1.clone(), step2.clone()]);

    in_memory
        .store_recipe(&mut recipe)
        .expect("We expect the recpe to store successfully");
    assert!(recipe.id.is_some());

    let recipes: Vec<Recipe> = in_memory
        .fetch_all_recipes()
        .expect("We expect to get recipes back out");
    assert_eq!(recipes.len(), 1);
    let recipe: Option<Recipe> = in_memory
        .fetch_recipe("my recipe")
        .expect("We expect the recipe to come back out");
    assert!(recipe.is_some());
    let recipe = recipe.unwrap();
    assert_eq!(recipe.title, "my recipe");
    assert_eq!(recipe.desc, "my description");
    assert_eq!(recipe.steps.len(), 2);
    let step1_got = &recipe.steps[0];
    let step2_got = &recipe.steps[1];
    assert_eq!(step1_got.prep_time, step1.prep_time);
    assert_eq!(step1_got.instructions, step1.instructions);
    assert_eq!(step1_got.ingredients.len(), step1.ingredients.len());
    assert_eq!(step1_got.ingredients[0], step1.ingredients[0]);
    assert_eq!(step1_got.ingredients[1], step1.ingredients[1]);
    assert_eq!(step2_got.prep_time, step2.prep_time);
    assert_eq!(step2_got.instructions, step2.instructions);
    assert_eq!(step2_got.ingredients.len(), step2.ingredients.len());
}

#[test]
fn test_fetch_recipe_ingredients() {
    let mut in_memory = init_sqlite_store!();
    let mut recipe = Recipe::new("my recipe", "my description");
    let mut step1 = Step::new(
        Some(std::time::Duration::from_secs(60 * 30)),
        "mix thoroughly",
    );
    step1.add_ingredients(vec![
        Ingredient::new("flour", None, Measure::cup(1.into()), "dry goods"),
        Ingredient::new(
            "salt",
            Some("Ground".to_owned()),
            Measure::tsp(1.into()),
            "seasoning",
        ),
    ]);
    let step2 = Step::new(None, "combine ingredients");
    recipe.add_steps(vec![step1.clone(), step2.clone()]);

    in_memory
        .store_recipe(&mut recipe)
        .expect("We expect the recpe to store successfully");

    let ingredients = in_memory
        .fetch_recipe_ingredients(recipe.id.unwrap())
        .expect("We expect to fetch ingredients for the recipe");
    assert_eq!(ingredients.len(), 2);
    assert_eq!(ingredients, step1.ingredients);
}
