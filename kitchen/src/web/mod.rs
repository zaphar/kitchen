// Copyright 2022 Jeremy Wall
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
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::{boxed, Full},
    extract::{Extension, Path},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, Router},
};
use mime_guess;
use recipe_store::{self, RecipeEntry, RecipeStore};
use rust_embed::RustEmbed;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, instrument};

use session::AuthStore;

mod auth;
mod session;

#[derive(RustEmbed)]
#[folder = "../web/dist"]
struct UiAssets;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match UiAssets::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}

#[instrument]
async fn ui_static_assets(Path(path): Path<String>) -> impl IntoResponse {
    info!("Serving ui path");

    let mut path = path.trim_start_matches("/");
    path = match path {
        "" | "inventory" | "plan" | "cook" | "login" => "index.html",
        _ => path,
    };
    debug!(path = path, "Serving transformed path");
    StaticFile(path.to_owned())
}

#[instrument]
async fn api_recipes(
    Extension(store): Extension<Arc<recipe_store::AsyncFileStore>>,
    Extension(app_store): Extension<Arc<session::SqliteStore>>,
    session: session::UserIdFromSession,
) -> impl IntoResponse {
    // Select recipes based on the user-id if it exists or serve the default if it does not.
    use session::{UserId, UserIdFromSession::*};
    let result = match session {
        NoUserId => store
            .get_recipes()
            .await
            .map_err(|e| format!("Error: {:?}", e)),
        FoundUserId(UserId(id)) => app_store
            .get_recipes_for_user(id.as_str())
            .await
            .map_err(|e| format!("Error: {:?}", e)),
    };
    match result {
        Ok(Some(recipes)) => Ok(axum::Json::from(recipes)),
        Ok(None) => Ok(axum::Json::from(Vec::<RecipeEntry>::new())),
        Err(e) => Err(e),
    }
}

#[instrument]
async fn api_categories(
    Extension(store): Extension<Arc<recipe_store::AsyncFileStore>>,
    Extension(app_store): Extension<Arc<session::SqliteStore>>,
    session: session::UserIdFromSession,
) -> impl IntoResponse {
    // Select Categories based on the user-id if it exists or serve the default if it does not.
    use session::{UserId, UserIdFromSession::*};
    let categories_result = match session {
        NoUserId => store
            .get_categories()
            .await
            .map_err(|e| format!("Error: {:?}", e)),
        FoundUserId(UserId(id)) => app_store
            .get_categories_for_user(id.as_str())
            .await
            .map_err(|e| format!("Error: {:?}", e)),
    };
    let result: Result<axum::Json<String>, String> = match categories_result {
        Ok(Some(categories)) => Ok(axum::Json::from(categories)),
        Ok(None) => Ok(axum::Json::from(String::new())),
        Err(e) => Err(e),
    };
    result
}

pub async fn add_user(store_path: PathBuf, username: String, password: String) {
    let app_store = session::SqliteStore::new(store_path)
        .await
        .expect("Unable to create app_store");
    let user_creds = session::UserCreds {
        id: session::UserId(username),
        pass: secrecy::Secret::from(password),
    };
    app_store
        .store_user_creds(user_creds)
        .await
        .expect("Failed to store user creds");
}

#[instrument(fields(recipe_dir=?recipe_dir_path,listen=?listen_socket), skip_all)]
pub async fn ui_main(recipe_dir_path: PathBuf, store_path: PathBuf, listen_socket: SocketAddr) {
    let store = Arc::new(recipe_store::AsyncFileStore::new(recipe_dir_path.clone()));
    //let dir_path = (&dir_path).clone();
    let app_store = session::SqliteStore::new(store_path)
        .await
        .expect("Unable to create app_store");
    let router = Router::new()
        .route("/", get(|| async { Redirect::temporary("/ui/plan") }))
        .route("/ui/*path", get(ui_static_assets))
        // recipes api path route
        .route("/api/v1/recipes", get(api_recipes))
        // categories api path route
        .route("/api/v1/categories", get(api_categories))
        // All the routes above require a UserId.
        .route("/api/v1/auth", get(auth::handler).post(auth::handler))
        // NOTE(jwall): Note that the layers are applied to the preceding routes not
        // the following routes.
        .layer(
            // NOTE(jwall): However service builder will apply the layers from top
            // to bottom.
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(Extension(store))
                .layer(Extension(app_store)),
        );
    info!(
        http = format!("http://{}", listen_socket),
        "Starting server"
    );
    axum::Server::bind(&listen_socket)
        .serve(router.into_make_service())
        .await
        .expect("Failed to start service");
}
