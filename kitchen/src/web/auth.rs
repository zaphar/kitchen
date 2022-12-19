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
use std::str::FromStr;
use std::sync::Arc;

use async_session::{Session, SessionStore};
use axum::{
    extract::Extension,
    http::{header, HeaderMap, StatusCode},
};
use axum_auth::AuthBasic;
use cookie::{Cookie, SameSite};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument};

use super::storage::{self, AuthStore, UserCreds};

// FIXME(jwall): This needs to live in a client integration library.
#[derive(Serialize, Deserialize)]
pub enum AccountResponse {
    Success { user_id: String },
    Err { message: String },
}

impl From<UserCreds> for AccountResponse {
    fn from(auth: UserCreds) -> Self {
        Self::Success {
            user_id: auth.user_id().to_owned(),
        }
    }
}

impl<'a> From<&'a str> for AccountResponse {
    fn from(msg: &'a str) -> Self {
        Self::Err {
            message: msg.to_string(),
        }
    }
}

#[instrument(skip_all, fields(user=%auth.0.0))]
pub async fn handler(
    auth: AuthBasic,
    Extension(session_store): Extension<Arc<storage::SqliteStore>>,
) -> (StatusCode, HeaderMap, axum::Json<AccountResponse>) {
    // NOTE(jwall): It is very important that you do **not** log the password
    // here. We convert the AuthBasic into UserCreds immediately to help prevent
    // that. Do not circumvent that protection.
    let auth = storage::UserCreds::from(auth);
    info!("Handling authentication request");
    let mut headers = HeaderMap::new();
    if let Ok(true) = session_store.check_user_creds(&auth).await {
        debug!("successfully authenticated user");
        // 1. Create a session identifier.
        let mut session = Session::new();
        if let Err(err) = session.insert("user_id", auth.user_id()) {
            error!(?err, "Unable to insert user id into session");
            let resp: AccountResponse = "Unable to insert user id into session".into();
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                axum::Json::from(resp),
            );
        }
        // 2. Store the session in the store.
        let cookie_value = match session_store.store_session(session).await {
            Err(err) => {
                error!(?err, "Unable to store session in session store");
                let resp: AccountResponse = "Unable to store session in session store".into();
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    headers,
                    axum::Json::from(resp),
                );
            }
            Ok(None) => {
                error!("Unable to create session cookie");
                let resp: AccountResponse = "Unable to create session cookie".into();
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    headers,
                    axum::Json::from(resp),
                );
            }
            Ok(Some(value)) => value,
        };
        // 3. Construct the Session Cookie.
        let cookie = Cookie::build(storage::AXUM_SESSION_COOKIE_NAME, cookie_value)
            .same_site(SameSite::Strict)
            .secure(true)
            .path("/")
            .finish();
        let parsed_cookie = match cookie.to_string().parse() {
            Err(err) => {
                error!(?err, "Unable to parse session cookie");
                let resp: AccountResponse = "Unable to parse session cookie".into();
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    headers,
                    axum::Json::from(resp),
                );
            }
            Ok(parsed_cookie) => parsed_cookie,
        };
        headers.insert(header::SET_COOKIE, parsed_cookie);
        // Respond with 200 OK
        let resp: AccountResponse = auth.into();
        (StatusCode::OK, headers, axum::Json::from(resp))
    } else {
        debug!("Invalid credentials");
        let headers = HeaderMap::new();
        let resp: AccountResponse = "Invalid user id or password".into();
        (StatusCode::UNAUTHORIZED, headers, axum::Json::from(resp))
    }
}

impl From<AuthBasic> for storage::UserCreds {
    #[instrument(skip_all)]
    fn from(AuthBasic((id, pass)): AuthBasic) -> Self {
        debug!(user = id, "Authorizing user");
        Self {
            id: storage::UserId(id),
            pass: match Secret::from_str(match pass {
                None => {
                    error!("No password provided in BasicAuth");
                    panic!("No password provided in BasicAuth");
                }
                Some(ref pass) => pass.as_str(),
            }) {
                Err(err) => {
                    error!("Unable to store pass in secret");
                    panic!("{}", err);
                }
                Ok(secret) => secret,
            },
        }
    }
}
