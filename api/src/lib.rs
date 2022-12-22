use std::collections::BTreeMap;

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
#[cfg(feature = "server")]
use axum::{
    self,
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
};
use serde::{Deserialize, Serialize};

use recipes::{IngredientKey, RecipeEntry};

#[derive(Serialize, Deserialize, Debug)]
pub enum Response<T> {
    Success(T),
    Err { status: u16, message: String },
    NotFound,
    Unauthorized,
}

impl<T> Response<T> {
    pub fn error<S: Into<String>>(code: u16, msg: S) -> Self {
        Self::Err {
            status: code,
            message: msg.into(),
        }
    }

    pub fn success(payload: T) -> Self {
        Self::Success(payload)
    }

    #[cfg(feature = "browser")]
    pub fn as_success(self) -> Option<T> {
        if let Self::Success(val) = self {
            Some(val)
        } else {
            None
        }
    }
}

#[cfg(feature = "server")]
impl<T> IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> AxumResponse {
        match &self {
            Self::Success(_) => (StatusCode::OK, axum::Json::from(self)).into_response(),
            Self::Err { status, message: _ } => {
                let code = match StatusCode::from_u16(*status) {
                    Ok(c) => c,
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (code, axum::Json::from(self)).into_response()
            }
            // TODO(jwall): Perhaps this can show a more useful json payload?
            Self::NotFound => (StatusCode::NOT_FOUND, axum::Json::from(self)).into_response(),
            Self::Unauthorized => {
                (StatusCode::UNAUTHORIZED, axum::Json::from(self)).into_response()
            }
        }
    }
}

impl<T> From<Result<T, String>> for Response<T> {
    fn from(val: Result<T, String>) -> Self {
        match val {
            Ok(val) => Response::Success(val),
            Err(e) => Response::error(500, e),
        }
    }
}

impl<T> From<Result<Option<T>, String>> for Response<T>
where
    T: Default,
{
    fn from(val: Result<Option<T>, String>) -> Self {
        match val {
            Ok(Some(val)) => Response::Success(val),
            Ok(None) => Response::Success(T::default()),
            Err(e) => Response::error(500, e),
        }
    }
}

pub type CategoryResponse = Response<String>;

pub type EmptyResponse = Response<()>;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserData {
    pub user_id: String,
}

pub type AccountResponse = Response<UserData>;

impl From<UserData> for AccountResponse {
    fn from(user_data: UserData) -> Self {
        Response::Success(user_data)
    }
}

pub type RecipeEntryResponse = Response<Vec<RecipeEntry>>;

impl From<Vec<RecipeEntry>> for RecipeEntryResponse {
    fn from(entries: Vec<RecipeEntry>) -> Self {
        Response::Success(entries)
    }
}

pub type PlanDataResponse = Response<Vec<(String, i32)>>;

impl From<Vec<(String, i32)>> for PlanDataResponse {
    fn from(plan: Vec<(String, i32)>) -> Self {
        Response::Success(plan)
    }
}

impl From<Option<Vec<(String, i32)>>> for PlanDataResponse {
    fn from(plan: Option<Vec<(String, i32)>>) -> Self {
        match plan {
            Some(plan) => Response::Success(plan),
            None => Response::Success(Vec::new()),
        }
    }
}

pub type PlanHistoryResponse = Response<BTreeMap<chrono::NaiveDate, Vec<(String, i32)>>>;

#[derive(Serialize, Deserialize)]
pub struct InventoryData {
    pub filtered_ingredients: Vec<IngredientKey>,
    pub modified_amts: Vec<(IngredientKey, String)>,
    pub extra_items: Vec<(String, String)>,
}

pub type InventoryResponse = Response<InventoryData>;

impl
    From<(
        Vec<IngredientKey>,
        Vec<(IngredientKey, String)>,
        Vec<(String, String)>,
    )> for InventoryData
{
    fn from(
        (filtered_ingredients, modified_amts, extra_items): (
            Vec<IngredientKey>,
            Vec<(IngredientKey, String)>,
            Vec<(String, String)>,
        ),
    ) -> Self {
        InventoryData {
            filtered_ingredients,
            modified_amts,
            extra_items,
        }
    }
}

impl From<InventoryData> for InventoryResponse {
    fn from(inventory_data: InventoryData) -> Self {
        Response::Success(inventory_data)
    }
}
