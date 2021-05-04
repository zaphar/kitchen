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
/// Storage backend for recipes.
use std::convert::From;
use std::path::Path;

use chrono::NaiveDate;
use rusqlite::{params, Connection, Result as SqliteResult, Transaction};
use uuid::Uuid;

use recipes::{unit::Measure, Ingredient, Mealplan, Recipe, Step};

// TODO Model the error domain of our storage layer.
#[derive(Debug)]
pub struct StorageError {
    message: String,
}

impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::SqliteFailure(e, msg) => StorageError {
                message: format!("{}: {}", e, msg.unwrap_or_default()),
            },
            rusqlite::Error::SqliteSingleThreadedMode => unimplemented!(),
            rusqlite::Error::FromSqlConversionFailure(_, _, _) => unimplemented!(),
            rusqlite::Error::IntegralValueOutOfRange(_, _) => todo!(),
            rusqlite::Error::Utf8Error(_) => todo!(),
            rusqlite::Error::NulError(_) => todo!(),
            rusqlite::Error::InvalidParameterName(_) => todo!(),
            rusqlite::Error::InvalidPath(_) => todo!(),
            rusqlite::Error::ExecuteReturnedResults => todo!(),
            rusqlite::Error::QueryReturnedNoRows => todo!(),
            rusqlite::Error::InvalidColumnIndex(_) => todo!(),
            rusqlite::Error::InvalidColumnName(_) => todo!(),
            rusqlite::Error::InvalidColumnType(_, _, _) => todo!(),
            rusqlite::Error::StatementChangedRows(_) => todo!(),
            rusqlite::Error::ToSqlConversionFailure(_) => todo!(),
            rusqlite::Error::InvalidQuery => todo!(),
            rusqlite::Error::MultipleStatement => todo!(),
            rusqlite::Error::InvalidParameterCount(_, _) => todo!(),
            _ => todo!(),
        }
    }
}

pub enum IterResult<Entity> {
    Some(Entity),
    Err(StorageError),
    None,
}

pub trait RecipeStore {
    fn store_mealplan(&self, plan: &Mealplan) -> Result<(), StorageError>;
    fn fetch_mealplan(&self, mealplan_id: Uuid) -> Result<Mealplan, StorageError>;
    fn fetch_mealplans_after_date(&self, date: NaiveDate) -> Result<Vec<Mealplan>, StorageError>;
    fn store_recipe(&self, e: &Recipe) -> Result<(), StorageError>;
    fn fetch_all_recipes(&self) -> Result<Vec<Recipe>, StorageError>;
    fn fetch_recipe_steps(&self, recipe_id: Uuid) -> Result<Option<Vec<Step>>, StorageError>;
    fn fetch_recipe_by_title(&self, k: &str) -> Result<Option<Recipe>, StorageError>;
    fn fetch_recipe_by_id(&self, k: Uuid) -> Result<Option<Recipe>, StorageError>;
    fn fetch_recipe_ingredients(&self, recipe_id: Uuid) -> Result<Vec<Ingredient>, StorageError>;
}

pub struct SqliteBackend {
    conn: Connection,
}

impl SqliteBackend {
    pub fn new<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }

    pub fn new_in_memory() -> SqliteResult<Self> {
        Ok(Self {
            conn: Connection::open_in_memory()?,
        })
    }

    pub fn get_schema_version(&self) -> SqliteResult<Option<u32>> {
        let mut stmt = self
            .conn
            .prepare("SELECT max(version) from schema_version")?;
        Ok(stmt.query_row([], |r| r.get(0))?)
    }

    pub fn start_transaction<'a>(&'a mut self) -> SqliteResult<TxHandle<'a>> {
        self.conn.transaction().map(|tx| TxHandle { tx })
    }

    pub fn create_schema(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version ( version INTEGER PRIMARY KEY )",
            [],
        )?;
        let version = self.get_schema_version()?;
        if let None = version {
            self.conn
                .execute("INSERT INTO schema_version ( version ) values ( 0 )", [])?;
        } else {
            return Ok(());
        }

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS mealplans (
                  id         BLOB PRIMARY KEY,
                  start_date TEXT
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS recipes (
                  id    BLOB PRIMARY KEY,
                  title TEXT UNIQUE NOT NULL,
                  desc  TEXT NOT NULL
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS mealplan_recipes (
                  plan_id    BLOB NOT NULL,
                  recipe_id  BLOB NOT NULL,
                  recipe_idx INTEGER NOT NULL,
                  FOREIGN KEY(plan_id) REFERENCES mealplans(id)
                  FOREIGN KEY(recipe_id) REFERENCES recipes(id)
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS steps (
                  recipe_id    BLOB NOT NULL,
                  step_idx     INTEGER NOT NULL,
                  prep_time    INTEGER, -- in seconds
                  instructions TEXT NOT NULL,
                  FOREIGN KEY(recipe_id) REFERENCES recipes(id)
                  CONSTRAINT step_key PRIMARY KEY (recipe_id, step_idx)
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS step_ingredients (
                  recipe_id      BLOB NOT NULL,
                  step_idx       INTEGER NOT NULL,
                  ingredient_idx INTEGER NOT NULL,
                  name           TEXT NOT NULL,
                  amt            TEXT NOT NULL,
                  category       TEXT NOT NULL,
                  form           TEXT,
                  FOREIGN KEY(recipe_id, step_idx) REFERENCES steps(recipe_id, step_idx),
                  CONSTRAINT step_ingredients_key PRIMARY KEY (recipe_id, step_idx, name, form)
            )",
            [],
        )?;
        Ok(())
    }
}

pub struct TxHandle<'conn> {
    tx: Transaction<'conn>,
}

impl<'conn> TxHandle<'conn> {
    pub fn serialize_step_stmt_rows(
        mut stmt: rusqlite::Statement,
        recipe_id: Uuid,
    ) -> Result<Option<Vec<Step>>, StorageError> {
        if let Ok(step_iter) = stmt.query_map(params![recipe_id], |row| {
            let prep_time: Option<i64> = row.get(2)?;
            let instructions: String = row.get(3)?;
            Ok(Step::new(
                prep_time.map(|i| std::time::Duration::from_secs(i as u64)),
                instructions,
            ))
        }) {
            let mut steps = Vec::new();
            for step in step_iter {
                steps.push(step?);
            }
            return Ok(Some(steps));
        }
        return Ok(None);
    }

    fn fill_recipe_steps(&self, recipe: &mut Recipe) -> Result<(), StorageError> {
        let stmt = self
            .tx
            .prepare("SELECT * FROM steps WHERE recipe_id = ?1 ORDER BY step_idx")?;
        let steps = Self::serialize_step_stmt_rows(stmt, recipe.id)?;
        let mut stmt = self.tx.prepare("SELECT * from step_ingredients WHERE recipe_id = ?1 and step_idx = ?2 ORDER BY ingredient_idx")?;
        if let Some(mut steps) = steps {
            for (step_idx, mut step) in steps.drain(0..).enumerate() {
                // TODO(jwall): Fetch the ingredients.
                let ing_iter = stmt.query_map(params![recipe.id, step_idx], |row| {
                    Self::map_ingredient_row(row)
                })?;
                for ing in ing_iter {
                    step.ingredients.push(ing?);
                }
                recipe.add_step(step);
            }
        }
        Ok(())
    }

    fn fill_plan_recipes(&self, plan: &mut Mealplan) -> Result<(), StorageError> {
        let mut stmt = self
            .tx
            .prepare("SELECT recipe_id from mealplan_recipes where plan_id = ?1")?;
        let id_iter = stmt.query_map(params![plan.id], |row| {
            let id: Uuid = row.get(0)?;
            Ok(id)
        })?;
        for id in id_iter {
            // TODO(jwall): A potential optimzation here is to do this in a single
            // select instead of a one at a time.
            if let Some(recipe) = self.fetch_recipe_by_id(id?)? {
                plan.recipes.push(recipe);
            }
        }
        Ok(())
    }

    fn map_recipe_row(r: &rusqlite::Row) -> Result<Recipe, rusqlite::Error> {
        let id: Uuid = r.get(0)?;
        let title: String = r.get(1)?;
        let desc: String = r.get(2)?;
        Ok(Recipe::new_id(id, title, desc))
    }

    fn map_ingredient_row(r: &rusqlite::Row) -> Result<Ingredient, rusqlite::Error> {
        let name: String = r.get(3)?;
        let amt: String = r.get(4)?;
        let category = r.get(5)?;
        let form = r.get(6)?;
        Ok(Ingredient::new(
            name,
            form,
            dbg!(Measure::parse(dbg!(&amt)).unwrap()),
            category,
        ))
    }
}

impl<'conn> RecipeStore for TxHandle<'conn> {
    fn fetch_all_recipes(&self) -> Result<Vec<Recipe>, StorageError> {
        let mut stmt = self.tx.prepare("SELECT * FROM recipes")?;
        let recipe_iter = stmt.query_map([], |r| Self::map_recipe_row(r))?;
        let mut recipes = Vec::new();
        for next in recipe_iter {
            recipes.push(next?);
        }
        Ok(recipes)
    }

    fn store_recipe(&self, recipe: &Recipe) -> Result<(), StorageError> {
        // If we don't have a transaction already we should start one.
        self.tx.execute(
            "INSERT OR REPLACE INTO recipes (id, title, desc) VALUES (?1, ?2, ?3)",
            params![recipe.id, recipe.title, recipe.desc],
        )?;
        for (idx, step) in recipe.steps.iter().enumerate() {
            self.tx.execute("INSERT INTO steps (recipe_id, step_idx, prep_time, instructions) VALUES (?1, ?2, ?3, ?4)",
            params![recipe.id, dbg!(idx), step.prep_time.map(|v| v.as_secs()) , step.instructions])?;
            for (ing_idx, ing) in step.ingredients.iter().enumerate() {
                dbg!(self.tx.execute(
                    "INSERT INTO step_ingredients (recipe_id, step_idx, ingredient_idx, name, amt, category, form) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![recipe.id, dbg!(idx), ing_idx, ing.name, format!("{}", ing.amt), ing.category, ing.form])?);
            }
        }
        Ok(())
    }

    fn store_mealplan(&self, plan: &Mealplan) -> Result<(), StorageError> {
        self.tx.execute(
            "INSERT OR REPLACE INTO mealplans (id, start_date) VALUES (?1, ?2)",
            params![plan.id, plan.start_date],
        )?;
        for (idx, recipe) in plan.recipes.iter().enumerate() {
            self.tx.execute(
                "INSERT INTO mealplan_recipes (plan_id, recipe_id, recipe_idx) VALUES (?1, ?2, ?3)",
                params![plan.id, recipe.id, idx],
            )?;
        }
        Ok(())
    }

    fn fetch_mealplan(&self, plan_id: Uuid) -> Result<Mealplan, StorageError> {
        let mut stmt = self.tx.prepare("SELECT * FROM mealplans WHERE id = ?1")?;
        let mut plan = stmt.query_row(params![plan_id], |row| {
            let id = row.get(0)?;
            let plan = Mealplan::new_id(id);
            if let Some(start_date) = dbg!(row.get(1)?) {
                Ok(plan.with_start_date(start_date))
            } else {
                Ok(plan)
            }
        })?;
        self.fill_plan_recipes(&mut plan)?;
        Ok(plan)
    }

    fn fetch_mealplans_after_date(&self, date: NaiveDate) -> Result<Vec<Mealplan>, StorageError> {
        let mut stmt = self
            .tx
            .prepare("SELECT * FROM mealplans WHERE start_date >= ?1 ORDER BY start_date DESC")?;
        let plan_iter = stmt.query_map(params![date], |row| {
            let id = row.get(0)?;
            let plan = Mealplan::new_id(id);
            if let Some(start_date) = dbg!(row.get(1)?) {
                Ok(plan.with_start_date(start_date))
            } else {
                Ok(plan)
            }
        })?;
        let mut plans = Vec::new();
        for plan in plan_iter {
            let mut plan = plan?;
            self.fill_plan_recipes(&mut plan)?;
            plans.push(plan);
        }
        Ok(plans)
    }

    fn fetch_recipe_steps(&self, recipe_id: Uuid) -> Result<Option<Vec<Step>>, StorageError> {
        let stmt = self
            .tx
            .prepare("SELECT * from steps WHERE recipe_id = ?1 ORDER BY step_idx")?;
        Self::serialize_step_stmt_rows(stmt, recipe_id)
    }

    fn fetch_recipe_by_id(&self, key: Uuid) -> Result<Option<Recipe>, StorageError> {
        let mut stmt = self.tx.prepare("SELECT * FROM recipes WHERE id = ?1")?;
        let recipe_iter = stmt.query_map(params![key], |r| Self::map_recipe_row(r))?;
        let mut recipe = recipe_iter
            .filter(|res| res.is_ok()) // TODO(jwall): What about failures here?
            .map(|r| r.unwrap())
            .next();
        // TODO(jwall): abstract this so it's shared between methods.
        if let Some(recipe) = recipe.as_mut() {
            self.fill_recipe_steps(recipe)?;
        }
        return Ok(recipe);
    }

    fn fetch_recipe_by_title(&self, key: &str) -> Result<Option<Recipe>, StorageError> {
        let mut stmt = self.tx.prepare("SELECT * FROM recipes WHERE title = ?1")?;
        let recipe_iter = stmt.query_map(params![key], |r| Self::map_recipe_row(r))?;
        let mut recipe = recipe_iter
            .filter(|res| res.is_ok()) // TODO(jwall): What about failures here?
            .map(|r| r.unwrap())
            .next();
        // TODO(jwall): abstract this so it's shared between methods.
        if let Some(recipe) = recipe.as_mut() {
            self.fill_recipe_steps(recipe)?;
        }
        return Ok(recipe);
    }

    fn fetch_recipe_ingredients(&self, recipe_id: Uuid) -> Result<Vec<Ingredient>, StorageError> {
        let mut stmt = self.tx.prepare(
            "SELECT * FROM step_ingredients WHERE recipe_id = ?1 ORDER BY step_idx, ingredient_idx",
        )?;
        let ing_iter = stmt.query_map(params![recipe_id], |row| Self::map_ingredient_row(row))?;
        let mut ingredients = Vec::new();
        for i in ing_iter {
            ingredients.push(i?);
        }
        Ok(ingredients)
    }
}

#[cfg(test)]
mod tests;
