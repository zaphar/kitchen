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
use tracing::error;
use web_sys::{window, Storage, Window};
use indexed_db::{self, Factory, Database};
use anyhow::{Result, Context};

pub async fn get_indexed_db(name: &str, version: Option<u32>) -> Result<Database<std::io::Error>> {
    let version = version.unwrap_or(0);
    let factory = Factory::<std::io::Error>::get().context("opening IndexedDB")?;
    let db = factory.open(name, version, |evt| async move {
        // NOTE(zaphar): This is the on upgradeneeded handler. It get's called on new databases or
        // database with an older version than the one we requested to build.
        let db = evt.database();
        // NOTE(jwall): This needs to be somewhat clever in handling version upgrades.
        if db.version() == 0 {
            // We use out of line keys for this object store
            db.build_object_store("store").create()?;
        }
        Ok(())
    }).await.context(format!("Openong or creating the database {}", name))?;
    Ok(db)
}

pub fn get_storage() -> Storage {
    get_window()
        .local_storage()
        .expect("Failed to get storage")
        .expect("No storage available")
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
