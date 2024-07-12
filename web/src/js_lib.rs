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
use anyhow::{Context, Result};
use indexed_db::{self, Database, Factory, Transaction};
use js_sys::Date;
use std::collections::HashSet;
use std::future::Future;
use tracing::error;
use web_sys::{window, Window};

pub fn get_storage() -> web_sys::Storage {
    get_window()
        .local_storage()
        .expect("Failed to get storage")
        .expect("No storage available")
}

pub const STATE_STORE_NAME: &'static str = "state-store";
pub const RECIPE_STORE_NAME: &'static str = "recipe-store";
pub const SERVING_COUNT_IDX: &'static str = "recipe-serving-count";
pub const CATEGORY_IDX: &'static str = "recipe-category";
pub const DB_VERSION: u32 = 1;

#[derive(Clone, Debug)]
pub struct DBFactory<'name> {
    name: &'name str,
    version: Option<u32>,
}

impl Default for DBFactory<'static> {
    fn default() -> Self {
        DBFactory {
            name: STATE_STORE_NAME,
            version: Some(DB_VERSION),
        }
    }
}

async fn version1_setup<'db>(
    stores: &HashSet<String>,
    db: &'db Database<std::io::Error>,
) -> Result<(), indexed_db::Error<std::io::Error>> {
    // We use out of line keys for this object store
    if !stores.contains(STATE_STORE_NAME) {
        db.build_object_store(STATE_STORE_NAME).create()?;
    }
    if !stores.contains(RECIPE_STORE_NAME) {
        let recipe_store = db.build_object_store(RECIPE_STORE_NAME).create()?;
        recipe_store
            .build_index(CATEGORY_IDX, "category")
            .create()?;
        recipe_store
            .build_index(SERVING_COUNT_IDX, "serving_count")
            .create()?;
    }
    Ok(())
}

impl<'name> DBFactory<'name> {
    pub async fn get_indexed_db(&self) -> Result<Database<std::io::Error>> {
        let factory = Factory::<std::io::Error>::get().context("opening IndexedDB")?;
        let db = factory
            .open(self.name, self.version.unwrap_or(0), |evt| async move {
                // NOTE(zaphar): This is the on upgradeneeded handler. It get's called on new databases or
                // databases with an older version than the one we requested to build.
                let db = evt.database();
                let stores = db
                    .object_store_names()
                    .into_iter()
                    .collect::<HashSet<String>>();
                // NOTE(jwall): This needs to be somewhat clever in handling version upgrades.
                if db.version() > 0 {
                    version1_setup(&stores, db).await?;
                }
                Ok(())
            })
            .await
            .context(format!("Opening or creating the database {}", self.name))?;
        Ok(db)
    }

    pub async fn rw_transaction<Fun, RetFut, Ret>(
        &self,
        stores: &[&str],
        transaction: Fun,
    ) -> indexed_db::Result<Ret, std::io::Error>
    where
        Fun: 'static + FnOnce(Transaction<std::io::Error>) -> RetFut,
        RetFut: 'static + Future<Output = indexed_db::Result<Ret, std::io::Error>>,
        Ret: 'static,
    {
        self.get_indexed_db()
            .await
            .expect("Failed to open database")
            .transaction(stores)
            .rw()
            .run(transaction)
            .await
    }

    pub async fn ro_transaction<Fun, RetFut, Ret>(
        &self,
        stores: &[&str],
        transaction: Fun,
    ) -> indexed_db::Result<Ret, std::io::Error>
    where
        Fun: 'static + FnOnce(Transaction<std::io::Error>) -> RetFut,
        RetFut: 'static + Future<Output = indexed_db::Result<Ret, std::io::Error>>,
        Ret: 'static,
    {
        self.get_indexed_db()
            .await
            .expect("Failed to open database")
            .transaction(stores)
            .run(transaction)
            .await
    }
}

pub fn get_ms_timestamp() -> u32 {
    Date::new_0().get_milliseconds()
}

pub fn get_window() -> Window {
    window().expect("No window present")
}

pub trait LogFailures<V, E> {
    fn swallow_and_log(self);
}

impl<E> LogFailures<(), E> for Result<(), E>
where
    E: std::fmt::Debug,
{
    fn swallow_and_log(self) {
        if let Err(e) = self {
            error!(err = ?e, "Error: ");
        }
    }
}
