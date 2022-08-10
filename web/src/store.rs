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
use async_trait::async_trait;
use std::sync::Arc;

use reqwasm;
use tracing::debug;

use recipe_store::RecipeStore;

#[cfg(target_arch = "wasm32")]
pub struct HttpStore {
    root: String,
}

#[cfg(target_arch = "wasm32")]
impl HttpStore {
    pub fn new(root: String) -> Self {
        Self { root }
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl RecipeStore<String> for HttpStore {
    async fn get_categories(&self) -> Result<Option<String>, String> {
        let mut path = self.root.clone();
        path.push_str("/categories");
        let resp = match reqwasm::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Error: {}", e)),
        };
        if resp.status() == 404 {
            debug!("Categories returned 404");
            Ok(None)
        } else if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()))
        } else {
            debug!("We got a valid response back!");
            let resp = resp.text().await;
            Ok(Some(resp.map_err(|e| format!("{}", e))?))
        }
    }

    async fn get_recipes(&self) -> Result<Option<Vec<String>>, String> {
        let mut path = self.root.clone();
        path.push_str("/recipes");
        let resp = match reqwasm::http::Request::get(&path).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Error: {}", e)),
        };
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()))
        } else {
            debug!("We got a valid response back!");
            Ok(resp.json().await.map_err(|e| format!("{}", e))?)
        }
    }
    //
}
