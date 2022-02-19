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
use std::path::PathBuf;

use async_std::fs::{read_dir, read_to_string, DirEntry};
use async_std::stream::StreamExt;
use static_dir::static_dir;
use warp::{http::StatusCode, hyper::Uri, Filter};

use crate::api::ParseError;

pub async fn get_recipes(recipe_dir_path: PathBuf) -> Result<Vec<String>, ParseError> {
    let mut entries = read_dir(recipe_dir_path).await?;
    let mut entry_vec = Vec::new();
    while let Some(res) = entries.next().await {
        let entry: DirEntry = res?;
        if entry.file_type().await?.is_dir() || entry.file_name().to_string_lossy() != "menu.txt" {
            // add it to the entry
            let recipe_contents = read_to_string(entry.path()).await?;
            entry_vec.push(recipe_contents);
        }
    }
    Ok(entry_vec)
}

pub async fn ui_main(recipe_dir_path: PathBuf) {
    let root = warp::path::end().map(|| warp::redirect::found(Uri::from_static("/ui")));
    let ui = warp::path("ui").and(static_dir!("../web/dist"));
    let api = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("recipes"))
        .then(move || {
            let recipe_dir_path = (&recipe_dir_path).clone();
            eprintln!("servicing api request.");
            async move {
                match get_recipes(recipe_dir_path).await {
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

    let routes = root.or(ui).or(api).with(warp::log("access log"));

    // TODO(jwall): Take listen address as an argument to this function instead.
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
