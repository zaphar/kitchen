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
use tower_http::trace::TraceLayer;
use tracing::{debug, info, instrument};

#[derive(RustEmbed)]
#[folder = "../web/dist"]
struct UiAssets;

pub struct StaticFile<T>(pub T)
where
    T: Into<String> + Clone;

impl<T> StaticFile<T>
where
    T: Into<String> + Clone,
{
    pub fn exists(&self) -> bool {
        UiAssets::get(self.0.clone().into().as_str()).is_some()
    }
}

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String> + Clone,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match UiAssets::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(&path).first_or_octet_stream();
                debug!(mime_type=%mime, %path, "request mime type determined");
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
async fn ui_assets(Path(path): Path<String>) -> impl IntoResponse {
    info!("Serving ui path");

    let path = path.trim_start_matches("/");
    debug!(path = path, "Serving transformed path");
    let file = StaticFile(path.to_owned());
    // TODO(jwall): We need to construct the entire html page here.
    // not just this split form.
    if file.exists() {
        file.into_response()
    } else {
        let index = UiAssets::get("index.html").expect("Unexpectedly can't find index.html");
        let body = boxed(Full::from(
            String::from_utf8_lossy(index.data.as_ref()).replace(
                "%kitchen-wasm",
                &kitchen_wasm::render_to_string(&format!("/ui/{}", path)),
            ),
        ));
        Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .body(body)
            .unwrap()
    }
}

#[instrument]
async fn api_recipes(Extension(store): Extension<Arc<recipe_store::AsyncFileStore>>) -> Response {
    let result: Result<axum::Json<Vec<RecipeEntry>>, String> = match store
        .get_recipes()
        .await
        .map_err(|e| format!("Error: {:?}", e))
    {
        Ok(Some(recipes)) => Ok(axum::Json::from(recipes)),
        Ok(None) => Ok(axum::Json::from(Vec::<RecipeEntry>::new())),
        Err(e) => Err(e),
    };
    result.into_response()
}

#[instrument]
async fn api_categories(
    Extension(store): Extension<Arc<recipe_store::AsyncFileStore>>,
) -> Response {
    let recipe_result = store
        .get_categories()
        .await
        .map_err(|e| format!("Error: {:?}", e));
    let result: Result<axum::Json<String>, String> = match recipe_result {
        Ok(Some(categories)) => Ok(axum::Json::from(categories)),
        Ok(None) => Ok(axum::Json::from(String::new())),
        Err(e) => Err(e),
    };
    result.into_response()
}

#[instrument(fields(recipe_dir=?recipe_dir_path,listen=?listen_socket), skip_all)]
pub async fn ui_main(recipe_dir_path: PathBuf, listen_socket: SocketAddr) {
    let store = Arc::new(recipe_store::AsyncFileStore::new(recipe_dir_path.clone()));
    //let dir_path = (&dir_path).clone();
    let router = Router::new()
        .route("/", get(|| async { Redirect::temporary("/ui/") }))
        .route("/ui/*path", get(ui_assets))
        // recipes api path route
        .route("/api/v1/recipes", get(api_recipes))
        // categories api path route
        .route("/api/v1/categories", get(api_categories))
        // NOTE(jwall): Note that the layers are applied to the preceding routes not
        // the following routes.
        .layer(TraceLayer::new_for_http())
        .layer(Extension(store));
    info!(
        http = format!("http://{}", listen_socket),
        "Starting server"
    );
    axum::Server::bind(&listen_socket)
        .serve(router.into_make_service())
        .await
        .expect("Failed to start service");
}
