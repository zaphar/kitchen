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
use std::path::Path;
use std::str::FromStr;

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
use ciborium;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::{
    self,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    SqlitePool,
};
use tracing::{debug, error, info, instrument};

use recipe_store::RecipeEntry;

mod error;

pub use error::*;

pub const AXUM_SESSION_COOKIE_NAME: &'static str = "kitchen-session-cookie";

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

    async fn get_recipes_for_user(&self, user_id: &str) -> Result<Option<Vec<RecipeEntry>>>;

    async fn store_recipes_for_user(&self, user_id: &str, recipes: &Vec<RecipeEntry>)
        -> Result<()>;

    async fn store_categories_for_user(&self, user_id: &str, categories: &str) -> Result<()>;
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
        let url = format!("sqlite://{}/store.db", path.as_ref().to_string_lossy());
        let options = SqliteConnectOptions::from_str(&url)?.journal_mode(SqliteJournalMode::Wal);
        info!(?options, "Connecting to sqlite db");
        let pool = Arc::new(sqlx::SqlitePool::connect_with(options).await?);
        Ok(Self { pool, url })
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

    async fn get_recipes_for_user(&self, user_id: &str) -> Result<Option<Vec<RecipeEntry>>> {
        // NOTE(jwall): We allow dead code becaue Rust can't figure out that
        // this code is actually constructed but it's done via the query_as
        // macro.
        #[allow(dead_code)]
        struct RecipeRow {
            pub recipe_id: String,
            pub recipe_text: Option<String>,
        }
        let rows = sqlx::query_as!(
            RecipeRow,
            "select recipe_id, recipe_text from recipes where user_id = ?",
            user_id,
        )
        .fetch_all(self.pool.as_ref())
        .await?
        .iter()
        .map(|row| {
            RecipeEntry(
                row.recipe_id.clone(),
                row.recipe_text.clone().unwrap_or_else(|| String::new()),
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
            sqlx::query!(
                "insert into recipes (user_id, recipe_id, recipe_text) values (?, ?, ?)",
                user_id,
                recipe_id,
                recipe_text,
            )
            .execute(self.pool.as_ref())
            .await?;
        }
        Ok(())
    }

    async fn store_categories_for_user(&self, user_id: &str, categories: &str) -> Result<()> {
        sqlx::query!(
            "insert into categories (user_id, category_text) values (?, ?)",
            user_id,
            categories,
        )
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}