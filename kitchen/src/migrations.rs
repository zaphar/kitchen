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
use sqlx::{migrate, SqlitePool};
use std::sync::Arc;

pub async fn run_migration(pool: Arc<SqlitePool>) {
    sqlx::migrate!("./migrations")
        .run(pool.as_ref())
        .await
        .expect("Unable to run migratins");
}
