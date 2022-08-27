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

use async_session::SessionStore;
use axum::{extract::Extension, http::StatusCode, response::IntoResponse};
use axum_auth::AuthBasic;
use secrecy::Secret;

use super::session;

pub async fn handler(
    auth: AuthBasic,
    Extension(session_store): Extension<session::RocksdbInnerStore>,
) -> impl IntoResponse {
    if let Ok(true) = session_store.check_user_creds(session::UserCreds::from(auth)) {
        // TODO(jwall): set up session for them
        // and redirect to the UI.
        todo!()
    } else {
        (StatusCode::UNAUTHORIZED, "Invalid user id or password")
    }
}

impl From<AuthBasic> for session::UserCreds {
    fn from(AuthBasic((id, pass)): AuthBasic) -> Self {
        Self {
            id: session::UserId(id),
            pass: Secret::from_str(pass.unwrap().as_str()).unwrap(),
        }
    }
}
