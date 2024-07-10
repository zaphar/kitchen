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
use async_std::sync::Arc;
use std::collections::BTreeSet;
use std::str::FromStr;
use std::{collections::BTreeMap, path::Path};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_session::{Session, SessionStore};
use async_trait::async_trait;
use axum::{
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    headers::Cookie,
    http::StatusCode,
};
use chrono::NaiveDate;
use ciborium;
use recipes::{IngredientKey, RecipeEntry};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::{
    self,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    SqlitePool,
};
use tracing::{debug, error, info, instrument};

mod error;
pub mod file_store;

pub use error::*;

pub const AXUM_SESSION_COOKIE_NAME: &'static str = "kitchen-session-cookie";

// TODO(jwall): Should this move to the recipe crate?
#[derive(Debug, Serialize, Deserialize)]
pub struct UserId(pub String);

#[derive(Debug)]
pub enum UserIdFromSession {
    FoundUserId(UserId),
    NoUserId,
}

pub struct UserCreds {
    pub id: UserId,
    pub pass: Secret<String>,
}

impl UserCreds {
    pub fn user_id(&self) -> &str {
        self.id.0.as_str()
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn make_id_key(cookie_value: &str) -> async_session::Result<String> {
    debug!("deserializing cookie");
    Ok(Session::id_from_cookie_value(cookie_value)?)
}

#[instrument(skip_all, fields(hash=payload))]
fn check_pass(payload: &String, pass: &Secret<String>) -> bool {
    let parsed_hash = PasswordHash::new(&payload).expect("Invalid Password Hash");
    debug!(password_hash=?parsed_hash, "successfuly obtained password hash");
    let check = Argon2::default().verify_password(pass.expose_secret().as_bytes(), &parsed_hash);
    if let Err(err) = &check {
        debug!(err=?err, "Couldn't verify password");
        return false;
    }
    check.is_ok()
}

#[async_trait]
pub trait APIStore {
    async fn get_categories_for_user(&self, user_id: &str) -> Result<Option<String>>;

    async fn get_category_mappings_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<Vec<(String, String)>>>;

    async fn save_category_mappings_for_user(
        &self,
        user_id: &str,
        mappings: &Vec<(String, String)>,
    ) -> Result<()>;

    async fn get_recipes_for_user(&self, user_id: &str) -> Result<Option<Vec<RecipeEntry>>>;

    async fn delete_recipes_for_user(&self, user_id: &str, recipes: &Vec<String>) -> Result<()>;

    async fn store_recipes_for_user(&self, user_id: &str, recipes: &Vec<RecipeEntry>)
        -> Result<()>;

    async fn store_categories_for_user(&self, user_id: &str, categories: &str) -> Result<()>;

    async fn get_recipe_entry_for_user<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        id: S,
    ) -> Result<Option<RecipeEntry>>;

    async fn fetch_latest_meal_plan<S: AsRef<str> + Send>(
        &self,
        user_id: S,
    ) -> Result<Option<Vec<(String, i32)>>>;

    async fn fetch_meal_plan_for_date<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: NaiveDate,
    ) -> Result<Option<Vec<(String, i32)>>>;

    async fn fetch_meal_plans_since<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: NaiveDate,
    ) -> Result<Option<BTreeMap<NaiveDate, Vec<(String, i32)>>>>;

    async fn fetch_all_meal_plans<S: AsRef<str> + Send>(
        &self,
        user_id: S,
    ) -> Result<Option<Vec<NaiveDate>>>;

    async fn delete_meal_plan_for_date<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: NaiveDate,
    ) -> Result<()>;

    async fn save_meal_plan<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        recipe_counts: &Vec<(String, i32)>,
        date: NaiveDate,
    ) -> Result<()>;

    async fn fetch_inventory_for_date<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: NaiveDate,
    ) -> Result<(
        Vec<IngredientKey>,
        Vec<(IngredientKey, String)>,
        Vec<(String, String)>,
    )>;

    async fn fetch_latest_inventory_data<S: AsRef<str> + Send>(
        &self,
        user_id: S,
    ) -> Result<(
        Vec<IngredientKey>,
        Vec<(IngredientKey, String)>,
        Vec<(String, String)>,
    )>;

    async fn save_inventory_data_for_date<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: &NaiveDate,
        filtered_ingredients: BTreeSet<IngredientKey>,
        modified_amts: BTreeMap<IngredientKey, String>,
        extra_items: Vec<(String, String)>,
    ) -> Result<()>;

    async fn save_inventory_data<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        filtered_ingredients: BTreeSet<IngredientKey>,
        modified_amts: BTreeMap<IngredientKey, String>,
        extra_items: Vec<(String, String)>,
    ) -> Result<()>;

    async fn fetch_staples<S: AsRef<str> + Send>(&self, user_id: S) -> Result<Option<String>>;

    async fn save_staples<S: AsRef<str> + Send>(&self, user_id: S, content: S) -> Result<()>;
}

#[async_trait]
pub trait AuthStore: SessionStore {
    /// Check user credentials against the user store.
    async fn check_user_creds(&self, user_creds: &UserCreds) -> Result<bool>;

    /// Insert or update user credentials in the user store.
    async fn store_user_creds(&self, user_creds: UserCreds) -> Result<()>;
}

#[async_trait]
impl<B> FromRequest<B> for UserIdFromSession
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    #[instrument(skip_all)]
    async fn from_request(req: &mut RequestParts<B>) -> std::result::Result<Self, Self::Rejection> {
        let Extension(session_store) = Extension::<Arc<SqliteStore>>::from_request(req)
            .await
            .expect("No Session store configured!");
        let cookies = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .expect("Unable to get headers fromrequest");
        // TODO(jwall): We should really validate the expiration and such on this cookie.
        if let Some(session_cookie) = cookies
            .as_ref()
            .and_then(|c| c.get(AXUM_SESSION_COOKIE_NAME))
        {
            debug!(?session_cookie, "processing session cookie");
            match session_store.load_session(session_cookie.to_owned()).await {
                Ok(Some(session)) => {
                    if let Some(user_id) = session.get::<UserId>("user_id") {
                        info!(user_id = user_id.0, "Found Authenticated session");
                        return Ok(Self::FoundUserId(user_id));
                    } else {
                        error!("No user id found in session");
                        return Ok(Self::NoUserId);
                    }
                }
                Ok(None) => {
                    debug!("no session defined in headers.");
                    return Ok(Self::NoUserId);
                }
                Err(e) => {
                    debug!(err=?e, "error deserializing session");
                    return Ok(Self::NoUserId);
                }
            }
        } else {
            debug!("no cookies defined in headers.");
            return Ok(Self::NoUserId);
        }
    }
}

#[derive(Clone, Debug)]
pub struct SqliteStore {
    pool: Arc<SqlitePool>,
    url: String,
}

impl SqliteStore {
    pub async fn new<P: AsRef<Path>>(path: P) -> sqlx::Result<Self> {
        std::fs::create_dir_all(&path)?;
        let url = format!("sqlite://{}/store.db", path.as_ref().to_string_lossy());
        let options = SqliteConnectOptions::from_str(&url)?
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true);
        info!(?options, "Connecting to sqlite db");
        let pool = Arc::new(sqlx::SqlitePool::connect_with(options).await?);
        Ok(Self { pool, url })
    }

    #[instrument(fields(conn_string=self.url), skip_all)]
    pub async fn run_migrations(&self) -> sqlx::Result<()> {
        info!("Running database migrations");
        sqlx::migrate!("./migrations")
            .run(self.pool.as_ref())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for SqliteStore {
    #[instrument(fields(conn_string=self.url), skip_all)]
    async fn load_session(&self, cookie_value: String) -> async_session::Result<Option<Session>> {
        let id = make_id_key(&cookie_value)?;
        debug!(id, "fetching session from sqlite");
        if let Some(payload) =
            sqlx::query_scalar!("select session_value from sessions where id = ?", id)
                .fetch_optional(self.pool.as_ref())
                .await?
        {
            debug!(sesion_id = id, "found session key");
            let session: Session = ciborium::de::from_reader(payload.as_slice())?;
            return Ok(Some(session));
        }
        return Ok(None);
    }

    #[instrument(fields(conn_string=self.url), skip_all)]
    async fn store_session(&self, session: Session) -> async_session::Result<Option<String>> {
        let id = session.id();
        let mut payload: Vec<u8> = Vec::new();
        ciborium::ser::into_writer(&session, &mut payload)?;
        sqlx::query!(
            "insert into sessions (id, session_value) values (?, ?)",
            id,
            payload
        )
        .execute(self.pool.as_ref())
        .await?;
        debug!(sesion_id = id, "successfully inserted session key");
        return Ok(session.into_cookie_value());
    }

    #[instrument(fields(conn_string=self.url), skip_all)]
    async fn destroy_session(&self, session: Session) -> async_session::Result {
        let id = session.id();
        sqlx::query!("delete from sessions where id = ?", id,)
            .execute(self.pool.as_ref())
            .await?;
        return Ok(());
    }

    #[instrument(fields(conn_string=self.url), skip_all)]
    async fn clear_store(&self) -> async_session::Result {
        sqlx::query!("delete from sessions")
            .execute(self.pool.as_ref())
            .await?;
        return Ok(());
    }
}

#[async_trait]
impl AuthStore for SqliteStore {
    #[instrument(fields(user=%user_creds.id.0, conn_string=self.url), skip_all)]
    async fn check_user_creds(&self, user_creds: &UserCreds) -> Result<bool> {
        let id = user_creds.user_id().to_owned();
        if let Some(payload) =
            sqlx::query_scalar!("select password_hashed from users where id = ?", id)
                .fetch_optional(self.pool.as_ref())
                .await?
        {
            debug!("Testing password for user");
            return Ok(check_pass(&payload, &user_creds.pass));
        }
        Ok(false)
    }

    #[instrument(fields(user=%user_creds.id.0, conn_string=self.url), skip_all)]
    async fn store_user_creds(&self, user_creds: UserCreds) -> Result<()> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(user_creds.pass.expose_secret().as_bytes(), &salt)
            .expect("failed to hash password");
        let id = user_creds.user_id().to_owned();
        let password_hashed = password_hash.to_string();
        debug!("adding password for user");
        sqlx::query!(
            "insert into users (id, password_hashed) values (?, ?)",
            id,
            password_hashed,
        )
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}

// TODO(jwall): We need to do some serious error modeling here.
#[async_trait]
impl APIStore for SqliteStore {
    async fn get_categories_for_user(&self, user_id: &str) -> Result<Option<String>> {
        match sqlx::query_scalar!(
            "select category_text from categories where user_id = ?",
            user_id,
        )
        .fetch_optional(self.pool.as_ref())
        .await?
        {
            Some(result) => Ok(result),
            None => Ok(None),
        }
    }

    async fn get_category_mappings_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<Vec<(String, String)>>> {
        struct Row {
            ingredient_name: String,
            category_name: String,
        }
        let rows: Vec<Row> = sqlx::query_file_as!(
            Row,
            "src/web/storage/fetch_category_mappings_for_user.sql",
            user_id
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        if rows.is_empty() {
            Ok(None)
        } else {
            let mut mappings = Vec::new();
            for r in rows {
                mappings.push((r.ingredient_name, r.category_name));
            }
            Ok(Some(mappings))
        }
    }

    async fn save_category_mappings_for_user(
        &self,
        user_id: &str,
        mappings: &Vec<(String, String)>,
    ) -> Result<()> {
        for (name, category) in mappings.iter() {
            sqlx::query_file!(
                "src/web/storage/save_category_mappings_for_user.sql",
                user_id,
                name,
                category,
            )
            .execute(self.pool.as_ref())
            .await?;
        }
        Ok(())
    }

    async fn get_recipe_entry_for_user<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        id: S,
    ) -> Result<Option<RecipeEntry>> {
        let id = id.as_ref();
        let user_id = user_id.as_ref();
        let entry = sqlx::query!(
            "select recipe_id, recipe_text, category, serving_count from recipes where user_id = ? and recipe_id = ?",
            user_id,
            id,
        )
        .fetch_all(self.pool.as_ref())
        .await?
        .iter()
        .map(|row| {
            RecipeEntry(
                row.recipe_id.clone(),
                row.recipe_text.clone().unwrap_or_else(|| String::new()),
                row.category.clone(),
                row.serving_count.clone(),
            )
        })
        .nth(0);
        Ok(entry)
    }

    async fn get_recipes_for_user(&self, user_id: &str) -> Result<Option<Vec<RecipeEntry>>> {
        let rows = sqlx::query!(
            "select recipe_id, recipe_text, category, serving_count from recipes where user_id = ?",
            user_id,
        )
        .fetch_all(self.pool.as_ref())
        .await?
        .iter()
        .map(|row| {
            RecipeEntry(
                row.recipe_id.clone(),
                row.recipe_text.clone().unwrap_or_else(|| String::new()),
                row.category.clone(),
                row.serving_count.clone(),
            )
        })
        .collect();
        Ok(Some(rows))
    }

    async fn store_recipes_for_user(
        &self,
        user_id: &str,
        recipes: &Vec<RecipeEntry>,
    ) -> Result<()> {
        for entry in recipes {
            let recipe_id = entry.recipe_id().to_owned();
            let recipe_text = entry.recipe_text().to_owned();
            let category = entry.category();
            let serving_count = entry.serving_count();
            sqlx::query!(
                "insert into recipes (user_id, recipe_id, recipe_text, category, serving_count) values (?, ?, ?, ?, ?)
    on conflict(user_id, recipe_id) do update set recipe_text=excluded.recipe_text, category=excluded.category",
                user_id,
                recipe_id,
                recipe_text,
                category,
                serving_count,
            )
            .execute(self.pool.as_ref())
            .await?;
        }
        Ok(())
    }

    async fn delete_recipes_for_user(&self, user_id: &str, recipes: &Vec<String>) -> Result<()> {
        let mut transaction = self.pool.as_ref().begin().await?;
        for recipe_id in recipes {
            sqlx::query!(
                "delete from recipes where user_id = ? and recipe_id = ?",
                user_id,
                recipe_id,
            )
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn store_categories_for_user(&self, user_id: &str, categories: &str) -> Result<()> {
        sqlx::query!(
            "insert into categories (user_id, category_text) values (?, ?)
    on conflict(user_id) do update set category_text=excluded.category_text",
            user_id,
            categories,
        )
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn save_meal_plan<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        recipe_counts: &Vec<(String, i32)>,
        date: NaiveDate,
    ) -> Result<()> {
        let user_id = user_id.as_ref();
        let mut transaction = self.pool.as_ref().begin().await?;
        sqlx::query!(
            "delete from plan_recipes where user_id = ? and plan_date = ?",
            user_id,
            date,
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query_file!("src/web/storage/init_meal_plan.sql", user_id, date)
            .execute(&mut *transaction)
            .await?;
        for (id, count) in recipe_counts {
            sqlx::query_file!(
                "src/web/storage/save_meal_plan.sql",
                user_id,
                date,
                id,
                count
            )
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn fetch_all_meal_plans<S: AsRef<str> + Send>(
        &self,
        user_id: S,
    ) -> Result<Option<Vec<NaiveDate>>> {
        let user_id = user_id.as_ref();
        struct Row {
            pub plan_date: NaiveDate,
        }
        let rows = sqlx::query_file_as!(Row, r#"src/web/storage/fetch_all_plans.sql"#, user_id,)
            .fetch_all(self.pool.as_ref())
            .await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let mut result = Vec::new();
        for row in rows {
            let date: NaiveDate = row.plan_date;
            result.push(date);
        }
        Ok(Some(result))
    }

    async fn fetch_meal_plans_since<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: NaiveDate,
    ) -> Result<Option<BTreeMap<NaiveDate, Vec<(String, i32)>>>> {
        let user_id = user_id.as_ref();
        struct Row {
            pub plan_date: NaiveDate,
            pub recipe_id: String,
            pub count: i64,
        }
        // NOTE(jwall): It feels like I shouldn't have to use an override here
        // but I do because of the way sqlite does types and how that interacts
        // with sqlx's type inference machinery.
        let rows = sqlx::query_file_as!(
            Row,
            r#"src/web/storage/fetch_meal_plans_since.sql"#,
            user_id,
            date
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let mut result = BTreeMap::new();
        for row in rows {
            let (date, recipe_id, count): (NaiveDate, String, i64) =
                (row.plan_date, row.recipe_id, row.count);
            result
                .entry(date.clone())
                .or_insert_with(|| Vec::new())
                .push((recipe_id, count as i32));
        }
        Ok(Some(result))
    }

    #[instrument(skip_all, fields(user_id=user_id.as_ref(), date))]
    async fn delete_meal_plan_for_date<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: NaiveDate,
    ) -> Result<()> {
        debug!("Processing delete request");
        let user_id = user_id.as_ref();
        let mut transaction = self.pool.as_ref().begin().await?;
        sqlx::query!(
            "delete from plan_table where user_id = ? and plan_date = ?",
            user_id,
            date
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query!(
            "delete from plan_recipes where user_id = ? and plan_date = ?",
            user_id,
            date
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query!(
            "delete from filtered_ingredients where user_id = ? and plan_date = ?",
            user_id,
            date
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query!(
            "delete from modified_amts where user_id = ? and plan_date = ?",
            user_id,
            date
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query!(
            "delete from extra_items where user_id = ? and plan_date = ?",
            user_id,
            date
        )
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn fetch_meal_plan_for_date<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: NaiveDate,
    ) -> Result<Option<Vec<(String, i32)>>> {
        let user_id = user_id.as_ref();
        struct Row {
            pub plan_date: NaiveDate,
            pub recipe_id: String,
            pub count: i64,
        }
        // NOTE(jwall): It feels like I shouldn't have to use an override here
        // but I do because of the way sqlite does types and how that interacts
        // with sqlx's type inference machinery.
        let rows = sqlx::query_file_as!(
            Row,
            "src/web/storage/fetch_plan_for_date.sql",
            user_id,
            date
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let mut result = Vec::new();
        for row in rows {
            let (_, recipe_id, count): (NaiveDate, String, i64) =
                (row.plan_date, row.recipe_id, row.count);
            result.push((recipe_id, count as i32));
        }
        Ok(Some(result))
    }

    async fn fetch_latest_meal_plan<S: AsRef<str> + Send>(
        &self,
        user_id: S,
    ) -> Result<Option<Vec<(String, i32)>>> {
        let user_id = user_id.as_ref();
        struct Row {
            pub plan_date: NaiveDate,
            pub recipe_id: String,
            pub count: i64,
        }
        // NOTE(jwall): It feels like I shouldn't have to use an override here
        // but I do because of the way sqlite does types and how that interacts
        // with sqlx's type inference machinery.
        let rows =
            sqlx::query_file_as!(Row, "src/web/storage/fetch_latest_meal_plan.sql", user_id,)
                .fetch_all(self.pool.as_ref())
                .await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let mut result = Vec::new();
        for row in rows {
            let (_, recipe_id, count): (NaiveDate, String, i64) =
                (row.plan_date, row.recipe_id, row.count);
            result.push((recipe_id, count as i32));
        }
        Ok(Some(result))
    }

    async fn fetch_inventory_for_date<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: NaiveDate,
    ) -> Result<(
        Vec<IngredientKey>,
        Vec<(IngredientKey, String)>,
        Vec<(String, String)>,
    )> {
        let user_id = user_id.as_ref();
        struct FilteredIngredientRow {
            name: String,
            form: String,
            measure_type: String,
        }
        let filtered_ingredient_rows: Vec<FilteredIngredientRow> = sqlx::query_file_as!(
            FilteredIngredientRow,
            "src/web/storage/fetch_filtered_ingredients_for_date.sql",
            user_id,
            date,
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        let mut filtered_ingredients = Vec::new();
        for row in filtered_ingredient_rows {
            filtered_ingredients.push(IngredientKey::new(
                row.name,
                if row.form.is_empty() {
                    None
                } else {
                    Some(row.form)
                },
                row.measure_type,
            ));
        }
        struct ModifiedAmtRow {
            name: String,
            form: String,
            measure_type: String,
            amt: String,
        }
        let modified_amt_rows = sqlx::query_file_as!(
            ModifiedAmtRow,
            "src/web/storage/fetch_modified_amts_for_date.sql",
            user_id,
            date,
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        let mut modified_amts = Vec::new();
        for row in modified_amt_rows {
            modified_amts.push((
                IngredientKey::new(
                    row.name,
                    if row.form.is_empty() {
                        None
                    } else {
                        Some(row.form)
                    },
                    row.measure_type,
                ),
                row.amt,
            ));
        }
        pub struct ExtraItemRow {
            name: String,
            amt: String,
        }
        let extra_items_rows = sqlx::query_file_as!(
            ExtraItemRow,
            "src/web/storage/fetch_extra_items_for_date.sql",
            user_id,
            date,
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        let mut extra_items = Vec::new();
        for row in extra_items_rows {
            extra_items.push((row.name, row.amt));
        }
        Ok((filtered_ingredients, modified_amts, extra_items))
    }

    // TODO(jwall): Deprecated
    async fn fetch_latest_inventory_data<S: AsRef<str> + Send>(
        &self,
        user_id: S,
    ) -> Result<(
        Vec<IngredientKey>,
        Vec<(IngredientKey, String)>,
        Vec<(String, String)>,
    )> {
        let user_id = user_id.as_ref();
        struct FilteredIngredientRow {
            name: String,
            form: String,
            measure_type: String,
        }
        let filtered_ingredient_rows: Vec<FilteredIngredientRow> = sqlx::query_file_as!(
            FilteredIngredientRow,
            "src/web/storage/fetch_inventory_filtered_ingredients.sql",
            user_id
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        let mut filtered_ingredients = Vec::new();
        for row in filtered_ingredient_rows {
            filtered_ingredients.push(IngredientKey::new(
                row.name,
                if row.form.is_empty() {
                    None
                } else {
                    Some(row.form)
                },
                row.measure_type,
            ));
        }
        struct ModifiedAmtRow {
            name: String,
            form: String,
            measure_type: String,
            amt: String,
        }
        let modified_amt_rows = sqlx::query_file_as!(
            ModifiedAmtRow,
            "src/web/storage/fetch_inventory_modified_amts.sql",
            user_id,
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        let mut modified_amts = Vec::new();
        for row in modified_amt_rows {
            modified_amts.push((
                IngredientKey::new(
                    row.name,
                    if row.form.is_empty() {
                        None
                    } else {
                        Some(row.form)
                    },
                    row.measure_type,
                ),
                row.amt,
            ));
        }
        pub struct ExtraItemRow {
            name: String,
            amt: String,
        }
        let extra_items_rows = sqlx::query_file_as!(
            ExtraItemRow,
            "src/web/storage/fetch_extra_items.sql",
            user_id,
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        let mut extra_items = Vec::new();
        for row in extra_items_rows {
            extra_items.push((row.name, row.amt));
        }
        Ok((filtered_ingredients, modified_amts, extra_items))
    }

    async fn save_inventory_data_for_date<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        date: &NaiveDate,
        filtered_ingredients: BTreeSet<IngredientKey>,
        modified_amts: BTreeMap<IngredientKey, String>,
        extra_items: Vec<(String, String)>,
    ) -> Result<()> {
        let user_id = user_id.as_ref();
        let mut transaction = self.pool.as_ref().begin().await?;
        // store the filtered_ingredients
        sqlx::query!(
            "delete from filtered_ingredients where user_id = ? and plan_date = ?",
            user_id,
            date
        )
        .execute(&mut *transaction)
        .await?;
        for key in filtered_ingredients {
            let name = key.name();
            let form = key.form();
            let measure_type = key.measure_type();
            sqlx::query_file!(
                "src/web/storage/save_filtered_ingredients_for_date.sql",
                user_id,
                name,
                form,
                measure_type,
                date,
            )
            .execute(&mut *transaction)
            .await?;
        }
        sqlx::query!(
            "delete from modified_amts where user_id = ? and plan_date = ?",
            user_id,
            date
        )
        .execute(&mut *transaction)
        .await?;
        // store the modified amts
        for (key, amt) in modified_amts {
            let name = key.name();
            let form = key.form();
            let measure_type = key.measure_type();
            let amt = &amt;
            sqlx::query_file!(
                "src/web/storage/save_modified_amts_for_date.sql",
                user_id,
                name,
                form,
                measure_type,
                amt,
                date,
            )
            .execute(&mut *transaction)
            .await?;
        }
        sqlx::query!(
            "delete from extra_items where user_id = ? and plan_date = ?",
            user_id,
            date
        )
        .execute(&mut *transaction)
        .await?;
        // Store the extra items
        for (name, amt) in extra_items {
            sqlx::query_file!(
                "src/web/storage/store_extra_items_for_date.sql",
                user_id,
                name,
                amt,
                date
            )
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn save_inventory_data<S: AsRef<str> + Send>(
        &self,
        user_id: S,
        filtered_ingredients: BTreeSet<IngredientKey>,
        modified_amts: BTreeMap<IngredientKey, String>,
        extra_items: Vec<(String, String)>,
    ) -> Result<()> {
        let user_id = user_id.as_ref();
        let mut transaction = self.pool.as_ref().begin().await?;
        // store the filtered_ingredients
        for key in filtered_ingredients {
            let name = key.name();
            let form = key.form();
            let measure_type = key.measure_type();
            sqlx::query_file!(
                "src/web/storage/save_inventory_filtered_ingredients.sql",
                user_id,
                name,
                form,
                measure_type,
            )
            .execute(&mut *transaction)
            .await?;
        }
        // store the modified amts
        for (key, amt) in modified_amts {
            let name = key.name();
            let form = key.form();
            let measure_type = key.measure_type();
            let amt = &amt;
            sqlx::query_file!(
                "src/web/storage/save_inventory_modified_amts.sql",
                user_id,
                name,
                form,
                measure_type,
                amt,
            )
            .execute(&mut *transaction)
            .await?;
        }
        // Store the extra items
        for (name, amt) in extra_items {
            sqlx::query_file!("src/web/storage/store_extra_items.sql", user_id, name, amt)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn save_staples<S: AsRef<str> + Send>(&self, user_id: S, content: S) -> Result<()> {
        let (user_id, content) = (user_id.as_ref(), content.as_ref());
        sqlx::query_file!("src/web/storage/save_staples.sql", user_id, content)
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }

    async fn fetch_staples<S: AsRef<str> + Send>(&self, user_id: S) -> Result<Option<String>> {
        let user_id = user_id.as_ref();
        if let Some(content) =
            sqlx::query_file_scalar!("src/web/storage/fetch_staples.sql", user_id)
                .fetch_optional(self.pool.as_ref())
                .await?
        {
            return Ok(Some(content));
        }
        Ok(None)
    }
}
