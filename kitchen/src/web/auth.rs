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
use secrecy::Secret;

use super::session;

pub async fn handler(
    auth: AuthBasic,
    Extension(session_store): Extension<session::RocksdbInnerStore>,
) -> impl IntoResponse {
    if let Ok(true) = session_store.check_user_creds(session::UserCreds::from(&auth)) {
        // TODO(jwall): set up session for them
        // and redirect to the UI.
        // 1. Create a session identifier.
        let mut session = Session::new();
        session.insert("user_id", auth.0).unwrap();
        let cookie_value = session_store.store_session(session).await.unwrap().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            header::SET_COOKIE,
            format!("{}={}", session::AXUM_SESSION_COOKIE_NAME, cookie_value)
                .parse()
                .unwrap(),
        );
        // 2. Store the session in the store.
        // 3. Construct the Session Cookie.
        (StatusCode::OK, headers, "Login Successful")
    } else {
        let headers = HeaderMap::new();
        (
            StatusCode::UNAUTHORIZED,
            headers,
            "Invalid user id or password",
        )
    }
}

impl<'a> From<&'a AuthBasic> for session::UserCreds {
    fn from(AuthBasic((id, pass)): &'a AuthBasic) -> Self {
        Self {
            id: session::UserId(id.clone()),
            pass: Secret::from_str(pass.clone().unwrap().as_str()).unwrap(),
        }
    }
}
