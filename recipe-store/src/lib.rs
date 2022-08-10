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

pub trait TenantStoreFactory<S, E>
where
    S: RecipeStore<E>,
    E: Send,
{
    fn get_user_store(&self, user: String) -> S;
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
/// Define the shared interface to use for interacting with a store of recipes.
pub trait RecipeStore<E>
where
    E: Send,
{
    // NOTE(jwall): For reasons I do not entirely understand yet
    // You have to specify that these are both Future + Send below
    // because the compiler can't figure it out for you.

    /// Get categories text unparsed.
    async fn get_categories(&self) -> Result<Option<String>, E>;
    /// Get list of recipe text unparsed.
    async fn get_recipes(&self) -> Result<Option<Vec<String>>, E>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
/// Define the shared interface to use for interacting with a store of recipes.
pub trait RecipeStore<E>
where
    E: Send,
{
    // NOTE(jwall): For reasons I do not entirely understand yet
    // You have to specify that these are both Future + Send below
    // because the compiler can't figure it out for you.

    /// Get categories text unparsed.
    async fn get_categories(&self) -> Result<Option<String>, E>;
    /// Get list of recipe text unparsed.
    async fn get_recipes(&self) -> Result<Option<Vec<String>>, E>;
}
