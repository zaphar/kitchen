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
use std::{future::Future, pin::Pin};

pub enum MaybeAsync<T>
where
    T: Send,
{
    Sync(T),
    // NOTE(jwall): For reasons I do not entirely understand yet
    // You have to specify that this is both Future + Send because
    // the compiler can't figure it out for you.
    Async(Pin<Box<dyn Future<Output = T> + Send>>),
}

impl<T> MaybeAsync<T>
where
    T: Send,
{
    pub async fn as_async(self) -> Result<T, &'static str> {
        use MaybeAsync::{Async, Sync};
        match self {
            Async(f) => Ok(f.await),
            Sync(_) => Err("Something went very wrong. Attempted to use Sync as Async."),
        }
    }
    pub fn as_sync(self) -> Result<T, &'static str> {
        use MaybeAsync::{Async, Sync};
        match self {
            Async(_) => Err("Something went very wrong. Attempted to use Async as Sync."),
            Sync(v) => Ok(v),
        }
    }
}

pub trait TenantStoreFactory<S, E>
where
    S: RecipeStore<E>,
    E: Send,
{
    fn get_user_store(&self, user: String) -> S;
}

pub trait RecipeStore<E>
where
    E: Send,
{
    /// Get categories text unparsed.
    fn get_categories(&self) -> MaybeAsync<Result<Option<String>, E>>;
    /// Get list of recipe text unparsed.
    fn get_recipes(&self) -> MaybeAsync<Result<Option<Vec<String>>, E>>;
}
