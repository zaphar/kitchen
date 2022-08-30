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
use cookie::{Cookie as CookieParse, SameSite};
#[cfg(feature = "rocksdb")]
use rocksdb::{
    BoundColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, MultiThreaded, Options,
};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument};

const SESSION_CF: &'static str = "kitchen_session";
const USER_CF: &'static str = "kitchen_users";

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
pub trait AuthStore: SessionStore {
    /// Check user credentials against the user store.
    async fn check_user_creds(&self, user_creds: &UserCreds) -> async_session::Result<bool>;

    /// Insert or update user credentials in the user store.
    async fn store_user_creds(&self, user_creds: UserCreds) -> async_session::Result<()>;
}

#[cfg(feature = "rocksdb")]
#[derive(Clone, Debug)]
pub struct RocksdbStore {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

#[cfg(feature = "rocksdb")]
impl RocksdbStore {
    pub fn new<P: AsRef<Path>>(name: P) -> Result<Self, rocksdb::Error> {
        let session_cf_opts = Options::default();
        let session_cf = ColumnFamilyDescriptor::new(SESSION_CF, session_cf_opts);
        let user_cf_opts = Options::default();
        let user_cf = ColumnFamilyDescriptor::new(USER_CF, user_cf_opts);
        let mut opts = Options::default();
        opts.create_missing_column_families(true);
        opts.create_if_missing(true);
        Ok(Self {
            db: Arc::new(DBWithThreadMode::open_cf_descriptors(
                &opts,
                name,
                vec![session_cf, user_cf],
            )?),
        })
    }

    fn get_session_column_family_handle(&self) -> Option<Arc<BoundColumnFamily>> {
        self.db.cf_handle(SESSION_CF)
    }

    fn get_users_column_family_handle(&self) -> Option<Arc<BoundColumnFamily>> {
        self.db.cf_handle(USER_CF)
    }
}

#[cfg(feature = "rocksdb")]
#[async_trait]
impl SessionStore for RocksdbStore {
    #[instrument]
    async fn load_session(&self, cookie_value: String) -> async_session::Result<Option<Session>> {
        let id = make_id_key(&cookie_value)?;
        let cf_handle = self
            .get_session_column_family_handle()
            .expect(&format!("column family {} is missing", SESSION_CF));
        if let Some(payload) = self.db.get_cf(&cf_handle, id.as_bytes())? {
            let session: Session = ciborium::de::from_reader(payload.as_slice())?;
            return Ok(Some(session));
        }
        Ok(None)
    }

    #[instrument]
    async fn store_session(&self, session: Session) -> async_session::Result<Option<String>> {
        let id = session.id();
        let mut payload: Vec<u8> = Vec::new();
        let cf_handle = self
            .get_session_column_family_handle()
            .expect(&format!("column family {} is missing", SESSION_CF));
        ciborium::ser::into_writer(&session, &mut payload)?;
        self.db
            .put_cf(&cf_handle, id.as_bytes(), payload.as_slice())?;
        Ok(session.into_cookie_value())
    }

    #[instrument]
    async fn destroy_session(&self, session: Session) -> async_session::Result {
        let id = session.id();
        let cf_handle = self
            .get_session_column_family_handle()
            .expect(&format!("column family {} is missing", SESSION_CF));
        self.db.delete_cf(&cf_handle, id.as_bytes())?;
        Ok(())
    }

    #[instrument]
    async fn clear_store(&self) -> async_session::Result {
        self.db.drop_cf(SESSION_CF)?;
        Ok(())
    }
}

#[cfg(feature = "rocksdb")]
#[async_trait]
impl AuthStore for RocksdbStore {
    #[instrument(fields(user=%user_creds.id.0), skip_all)]
    async fn check_user_creds(&self, user_creds: &UserCreds) -> async_session::Result<bool> {
        // TODO(jwall): Make this function asynchronous.
        info!("checking credentials for user");
        let cf_handle = self
            .get_users_column_family_handle()
            .expect(&format!("column family {} is missing", USER_CF));
        if let Some(payload) = self
            .db
            .get_cf(&cf_handle, user_creds.user_id().as_bytes())?
        {
            debug!("Found user in credential store");
            let payload = String::from_utf8_lossy(payload.as_slice()).to_string();
            return Ok(check_pass(&payload, &user_creds.pass));
        }
        Ok(false)
    }

    // TODO(jwall): Make this function asynchronous.
    #[instrument(fields(user=%user_creds.id.0), skip_all)]
    async fn store_user_creds(&self, user_creds: UserCreds) -> async_session::Result<()> {
        // TODO(jwall): Enforce a password length?
        // TODO(jwall): Make this function asynchronous.
        info!("storing credentials for user {}", user_creds.id.0);
        let cf_handle = self
            .get_users_column_family_handle()
            .expect(&format!("column family {} is missing", USER_CF));
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(user_creds.pass.expose_secret().as_bytes(), &salt)
            .expect("failed to hash password");
        self.db.put_cf(
            &cf_handle,
            user_creds.id.0.as_bytes(),
            password_hash.to_string().as_bytes(),
        )?;
        Ok(())
    }
}

#[async_trait]
impl<B> FromRequest<B> for UserIdFromSession
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    #[instrument(skip_all)]
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(session_store) = Extension::<SqliteStore>::from_request(req)
            .await
            .expect("No Session store configured!");
        let cookies = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .unwrap();
        if let Some(session_cookie) = cookies
            .as_ref()
            .and_then(|c| c.get(AXUM_SESSION_COOKIE_NAME))
        {
            debug!(?session_cookie, "processing session cookie");
            match session_store.load_session(session_cookie.to_owned()).await {
                Ok(Some(session)) => {
                    if let Some(user_id) = session.get::<UserId>("user_id") {
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

#[cfg(feature = "sqlite")]
use sqlx::{
    self,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    SqlitePool,
};
#[cfg(feature = "sqlite")]
use std::str::FromStr;

#[cfg(feature = "sqlite")]
#[derive(Clone, Debug)]
pub struct SqliteStore {
    pool: Arc<SqlitePool>,
    url: String,
}

#[cfg(feature = "sqlite")]
impl SqliteStore {
    pub async fn new<P: AsRef<Path>>(path: P) -> sqlx::Result<Self> {
        let url = format!("sqlite://{}/store.db", path.as_ref().to_string_lossy());
        let options = SqliteConnectOptions::from_str(&url)?.journal_mode(SqliteJournalMode::Wal);
        let pool = Arc::new(sqlx::SqlitePool::connect_with(options).await?);
        Ok(Self { pool, url })
    }
}

#[cfg(feature = "sqlite")]
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

#[cfg(feature = "sqlite")]
#[async_trait]
impl AuthStore for SqliteStore {
    #[instrument(fields(user=%user_creds.id.0, conn_string=self.url), skip_all)]
    async fn check_user_creds(&self, user_creds: &UserCreds) -> async_session::Result<bool> {
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
    async fn store_user_creds(&self, user_creds: UserCreds) -> async_session::Result<()> {
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
