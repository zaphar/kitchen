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

use async_session::{Session, SessionStore};
use axum::{
    extract::Extension,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_auth::AuthBasic;
use cookie::{Cookie, SameSite};
use secrecy::Secret;
use tracing::{debug, info, instrument};

use super::session::{self, AuthStore};

#[instrument(skip_all, fields(user=%auth.0.0))]
pub async fn handler(
    auth: AuthBasic,
    Extension(session_store): Extension<session::SqliteStore>,
) -> impl IntoResponse {
    // NOTE(jwall): It is very important that you do **not** log the password
    // here. We convert the AuthBasic into UserCreds immediately to help prevent
    // that. Do not circumvent that protection.
    let auth = session::UserCreds::from(auth);
    info!("Handling authentication request");
    if let Ok(true) = session_store.check_user_creds(&auth).await {
        debug!("successfully authenticated user");
        // 1. Create a session identifier.
        let mut session = Session::new();
        session.insert("user_id", auth.user_id()).unwrap();
        // 2. Store the session in the store.
        let cookie_value = session_store.store_session(session).await.unwrap().unwrap();
        let mut headers = HeaderMap::new();
        // 3. Construct the Session Cookie.
        let cookie = Cookie::build(session::AXUM_SESSION_COOKIE_NAME, cookie_value)
            .same_site(SameSite::Strict)
            .secure(true)
            .finish();
        headers.insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
        // Respond with 200 OK
        (StatusCode::OK, headers, "Login Successful")
    } else {
        debug!("Invalid credentials");
        let headers = HeaderMap::new();
        (
            StatusCode::UNAUTHORIZED,
            headers,
            "Invalid user id or password",
        )
    }
}

impl From<AuthBasic> for session::UserCreds {
    fn from(AuthBasic((id, pass)): AuthBasic) -> Self {
        Self {
            id: session::UserId(id.clone()),
            pass: Secret::from_str(pass.clone().unwrap().as_str()).unwrap(),
        }
    }
}
