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
#[cfg(not(target_arch = "wasm32"))]
use async_std::{
    fs::{read_dir, read_to_string, DirEntry, File},
    io::{self, ReadExt},
    path::PathBuf,
    stream::StreamExt,
};
use async_trait::async_trait;
#[cfg(target_arch = "wasm32")]
use reqwasm;
#[cfg(not(target_arch = "wasm32"))]
use tracing::warn;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct Error(String);

impl From<std::io::Error> for Error {
    fn from(item: std::io::Error) -> Self {
        Error(format!("{:?}", item))
    }
}

impl From<String> for Error {
    fn from(item: String) -> Self {
        Error(item)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(item: std::string::FromUtf8Error) -> Self {
        Error(format!("{:?}", item))
    }
}

#[cfg(target_arch = "wasm32")]
impl From<reqwasm::Error> for Error {
    fn from(item: reqwasm::Error) -> Self {
        Error(format!("{:?}", item))
    }
}

pub trait TenantStoreFactory<S>
where
    S: RecipeStore,
{
    fn get_user_store(&self, user: String) -> S;
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
/// Define the shared interface to use for interacting with a store of recipes.
pub trait RecipeStore: Clone + Sized {
    /// Get categories text unparsed.
    async fn get_categories(&self) -> Result<Option<String>, Error>;
    /// Get list of recipe text unparsed.
    async fn get_recipes(&self) -> Result<Option<Vec<String>>, Error>;
}

// NOTE(jwall): Futures in webassembly can't implement `Send` easily so we define
// this trait differently based on architecture.
#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
/// Define the shared interface to use for interacting with a store of recipes.
pub trait RecipeStore: Clone + Sized {
    /// Get categories text unparsed.
    async fn get_categories(&self) -> Result<Option<String>, Error>;
    /// Get list of recipe text unparsed.
    async fn get_recipes(&self) -> Result<Option<Vec<String>>, Error>;
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct AsyncFileStore {
    path: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl AsyncFileStore {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { path: root.into() }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
// TODO(jwall): We need to model our own set of errors for this.
impl RecipeStore for AsyncFileStore {
    #[instrument(skip_all)]
    async fn get_categories(&self) -> Result<Option<String>, Error> {
        let mut category_path = PathBuf::new();
        category_path.push(&self.path);
        category_path.push("categories.txt");
        let category_file = File::open(&category_path).await?;
        debug!(category_file = ?category_path, "Opened category file");
        let mut buf_reader = io::BufReader::new(category_file);
        let mut contents = Vec::new();
        buf_reader.read_to_end(&mut contents).await?;
        Ok(Some(String::from_utf8(contents)?))
    }

    async fn get_recipes(&self) -> Result<Option<Vec<String>>, Error> {
        let mut recipe_path = PathBuf::new();
        recipe_path.push(&self.path);
        recipe_path.push("recipes");
        let mut entries = read_dir(&recipe_path).await?;
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
                debug!("adding recipe file {}", entry.file_name().to_string_lossy());
                let recipe_contents = read_to_string(entry.path()).await?;
                entry_vec.push(recipe_contents);
            } else {
                warn!(
                    file = %entry.path().to_string_lossy(),
                    "skipping file not a recipe",
                );
            }
        }
        Ok(Some(entry_vec))
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
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
impl RecipeStore for HttpStore {
    #[instrument]
    async fn get_categories(&self) -> Result<Option<String>, Error> {
        let mut path = self.root.clone();
        path.push_str("/categories");
        let resp = reqwasm::http::Request::get(&path).send().await?;
        if resp.status() == 404 {
            debug!("Categories returned 404");
            Ok(None)
        } else if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            let resp = resp.text().await;
            Ok(Some(resp.map_err(|e| format!("{}", e))?))
        }
    }

    #[instrument]
    async fn get_recipes(&self) -> Result<Option<Vec<String>>, Error> {
        let mut path = self.root.clone();
        path.push_str("/recipes");
        let resp = reqwasm::http::Request::get(&path).send().await?;
        if resp.status() != 200 {
            Err(format!("Status: {}", resp.status()).into())
        } else {
            debug!("We got a valid response back!");
            Ok(resp.json().await.map_err(|e| format!("{}", e))?)
        }
    }
    //
}
