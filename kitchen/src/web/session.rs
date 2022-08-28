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
use async_session::{async_trait, Session, SessionStore};
use axum::{
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    headers::Cookie,
    http::StatusCode,
};
use ciborium;
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

#[derive(Clone, Debug)]
pub struct RocksdbInnerStore {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl RocksdbInnerStore {
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

    fn make_id_key(cookie_value: &str) -> async_session::Result<String> {
        Ok(Session::id_from_cookie_value(cookie_value)?)
    }

    #[instrument(fields(user=%user_creds.id.0), skip_all)]
    pub fn check_user_creds(&self, user_creds: &UserCreds) -> async_session::Result<bool> {
        info!("checking credentials for user");
        let cf_handle = self
            .get_users_column_family_handle()
            .expect(&format!("column family {} is missing", USER_CF));
        if let Some(payload) = self.db.get_cf(&cf_handle, user_creds.id.0.as_bytes())? {
            debug!("Found user in credential store");
            let payload = String::from_utf8_lossy(payload.as_slice()).to_string();
            let parsed_hash = PasswordHash::new(&payload).expect("Invalid Password Hash");
            debug!(password_hash=?parsed_hash, "successfuly obtained password hash");
            let check = Argon2::default()
                .verify_password(user_creds.pass.expose_secret().as_bytes(), &parsed_hash);
            if let Err(err) = &check {
                debug!(err=?err, "Couldn't verify password")
            }
            return Ok(check.is_ok());
        }
        Ok(false)
    }

    #[instrument(fields(user=%user_creds.id.0), skip_all)]
    pub fn store_user_creds(&self, user_creds: UserCreds) -> async_session::Result<()> {
        // TODO(jwall): Enforce a password length?
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
impl SessionStore for RocksdbInnerStore {
    #[instrument]
    async fn load_session(&self, cookie_value: String) -> async_session::Result<Option<Session>> {
        let id = Self::make_id_key(&cookie_value)?;
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

#[async_trait]
impl<B> FromRequest<B> for UserIdFromSession
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    #[instrument(skip_all)]
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(session_store) = Extension::<RocksdbInnerStore>::from_request(req)
            .await
            .expect("No Session store configured!");
        let cookies = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .unwrap();
        if let Some(session_cookie) = cookies
            .as_ref()
            .and_then(|c| c.get(AXUM_SESSION_COOKIE_NAME))
        {
            if let Some(session) = session_store
                .load_session(session_cookie.to_owned())
                .await
                .unwrap()
            {
                if let Some(user_id) = session.get::<UserId>("user_id") {
                    return Ok(Self::FoundUserId(user_id));
                } else {
                    error!("No user id found in session");
                    return Ok(Self::NoUserId);
                }
            } else {
                debug!("no session defined in headers.");
                return Ok(Self::NoUserId);
            }
        } else {
            debug!("no cookies defined in headers.");
            return Err((StatusCode::UNAUTHORIZED, "Authentication Required"));
        }
    }
}
