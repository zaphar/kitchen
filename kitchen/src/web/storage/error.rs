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

use sqlx::Error as SqliteErr;
use tracing::error;

#[derive(Debug, Clone)]
pub enum Error {
    IO(String),
    Protocol(String),
    BadQuery(String),
    Timeout,
    NoRecords,
    Configuration(String),
    MalformedData(String),
    InternalError(String),
}

impl From<SqliteErr> for Error {
    fn from(e: SqliteErr) -> Self {
        match e {
            SqliteErr::Configuration(e) => Error::Configuration(format!("{:?}", e)),
            SqliteErr::PoolTimedOut => Error::Timeout,
            SqliteErr::PoolClosed => Error::InternalError(format!("Pool Closed")),
            SqliteErr::WorkerCrashed => Error::InternalError(format!("Worker Crashed!")),
            SqliteErr::Database(e) => Error::InternalError(format!("{:?}", e)),
            SqliteErr::Io(e) => Error::IO(format!("{:?}", e)),
            SqliteErr::Tls(e) => Error::Protocol(format!("{:?}", e)),
            SqliteErr::Protocol(e) => Error::Protocol(format!("{:?}", e)),
            SqliteErr::RowNotFound => Error::NoRecords,
            SqliteErr::TypeNotFound { type_name } => {
                Error::BadQuery(format!("Type not found `{}`", type_name))
            }
            SqliteErr::ColumnIndexOutOfBounds { index, len } => {
                Error::BadQuery(format!("column index {} out of bounds for {}", index, len))
            }
            SqliteErr::ColumnNotFound(col) => {
                Error::BadQuery(format!("Column not found `{}`", col))
            }
            SqliteErr::ColumnDecode { index, source } => Error::MalformedData(format!(
                "Column index {} can't be decoded: {}",
                index, source
            )),
            SqliteErr::Decode(e) => Error::MalformedData(format!("Decode error: {}", e)),
            SqliteErr::Migrate(_) => todo!(),
            err => {
                error!(?err, "Unhandled Error type encountered");
                Error::InternalError(format!("Unhandled Error type encountered {:?}", err))
            }
        }
    }
}
