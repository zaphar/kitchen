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

use async_std::fs::{self, read_dir, read_to_string, DirEntry};
use async_std::stream::StreamExt;
use static_dir::static_dir;
use tracing::{info, instrument, warn};
use warp::{http::StatusCode, hyper::Uri, Filter};

use crate::api::ParseError;

#[instrument(fields(recipe_dir=?recipe_dir_path), skip_all)]
pub async fn get_recipes(recipe_dir_path: PathBuf) -> Result<Vec<String>, ParseError> {
    let mut entries = read_dir(recipe_dir_path).await?;
    let mut entry_vec = Vec::new();
    // Special files that we ignore when fetching recipes
    let filtered = vec!["menu.txt", "categories.txt"];
    while let Some(res) = entries.next().await {
        let entry: DirEntry = res?;

        if !entry.file_type().await?.is_dir()
            && !filtered
                .iter()
                .any(|&s| s == entry.file_name().to_string_lossy().to_string())
        {
            // add it to the entry
            info!("adding recipe file {}", entry.file_name().to_string_lossy());
            let recipe_contents = read_to_string(entry.path()).await?;
            entry_vec.push(recipe_contents);
        } else {
            warn!(
                file = %entry.path().to_string_lossy(),
                "skipping file not a recipe",
            );
        }
    }
    Ok(entry_vec)
}

#[instrument(fields(recipe_dir=?recipe_dir_path,listen=?listen_socket), skip_all)]
pub async fn ui_main(recipe_dir_path: PathBuf, listen_socket: SocketAddr) {
    let root = warp::path::end().map(|| warp::redirect::found(Uri::from_static("/ui")));
    let ui = warp::path("ui").and(static_dir!("../web/dist"));
    let dir_path = (&recipe_dir_path).clone();

    // recipes api path route
    let recipe_path = warp::path("recipes").then(move || {
        let dir_path = (&dir_path).clone();
        info!(?dir_path, "servicing recipe api request.");
        async move {
            match get_recipes(dir_path).await {
                Ok(recipes) => {
                    warp::reply::with_status(warp::reply::json(&recipes), StatusCode::OK)
                }
                Err(e) => warp::reply::with_status(
                    warp::reply::json(&format!("Error: {:?}", e)),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
            }
        }
    });

    // categories api path route
    let mut file_path = (&recipe_dir_path).clone();
    file_path.push("categories.txt");
    let categories_path = warp::path("categories").then(move || {
        info!(?file_path, "servicing category api request");
        let file_path = (&file_path).clone();
        async move {
            match fs::metadata(&file_path).await {
                Ok(_) => {
                    let content = read_to_string(&file_path).await.unwrap();
                    warp::reply::with_status(warp::reply::json(&content), StatusCode::OK)
                }
                Err(_) => warp::reply::with_status(
                    warp::reply::json(&"No categories found"),
                    StatusCode::NOT_FOUND,
                ),
            }
        }
    });
    let api = warp::path("api")
        .and(warp::path("v1"))
        .and(recipe_path.or(categories_path));

    let routes = root.or(ui).or(api).with(warp::log("access log"));

    warp::serve(routes).run(listen_socket).await;
}
