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
use async_std::{
    fs::{read_dir, read_to_string, DirEntry, File},
    io::{self, ReadExt},
    path::PathBuf,
    stream::StreamExt,
};
use async_trait::async_trait;

use tracing::{info, instrument, warn};

use recipe_store::RecipeStore;

pub struct AsyncFileStore {
    path: PathBuf,
}

impl AsyncFileStore {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { path: root.into() }
    }
}

#[async_trait]
// TODO(jwall): We need to model our own set of errors for this.
impl RecipeStore<io::Error> for AsyncFileStore {
    #[instrument(skip_all)]
    async fn get_categories(&self) -> Result<Option<String>, io::Error> {
        let mut category_path = PathBuf::new();
        category_path.push(&self.path);
        category_path.push("categories.txt");
        let category_file = match File::open(&category_path).await {
            Ok(f) => f,
            Err(e) => {
                if let io::ErrorKind::NotFound = e.kind() {
                    return Ok(None);
                }
                return Err(e);
            }
        };
        let mut buf_reader = io::BufReader::new(category_file);
        let mut contents = Vec::new();
        if let Err(e) = buf_reader.read_to_end(&mut contents).await {
            return Err(e);
        }
        match String::from_utf8(contents) {
            Ok(s) => Ok(Some(s)),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }

    async fn get_recipes(&self) -> Result<Option<Vec<String>>, io::Error> {
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
        Ok(Some(entry_vec))
    }
}
