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
use async_std::sync::Arc;
use std::path::Path;

use async_session::{async_trait, Session, SessionStore};
use ciborium;
use rocksdb::{
    BoundColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, MultiThreaded, Options,
};

const SESSION_CF: &'static str = "kitchen_session";

#[derive(Clone, Debug)]
pub struct RocksdbInnerStore {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl RocksdbInnerStore {
    pub fn new<P: AsRef<Path>>(name: P) -> Result<Self, rocksdb::Error> {
        let cf_opts = Options::default();
        let cf = ColumnFamilyDescriptor::new(SESSION_CF, cf_opts);
        let mut opts = Options::default();
        opts.create_missing_column_families(true);
        opts.create_if_missing(true);
        Ok(Self {
            db: Arc::new(DBWithThreadMode::open_cf_descriptors(
                &opts,
                name,
                vec![cf],
            )?),
        })
    }

    fn get_column_family_handle(&self) -> Option<Arc<BoundColumnFamily>> {
        self.db.cf_handle(SESSION_CF)
    }

    fn make_id_key(cookie_value: &str) -> async_session::Result<String> {
        Ok(Session::id_from_cookie_value(cookie_value)?)
    }
}

#[async_trait]
impl SessionStore for RocksdbInnerStore {
    async fn load_session(&self, cookie_value: String) -> async_session::Result<Option<Session>> {
        let id = Self::make_id_key(&cookie_value)?;
        let cf_handle = self
            .get_column_family_handle()
            .expect(&format!("column family {} is missing", SESSION_CF));
        if let Some(payload) = self.db.get_cf(&cf_handle, id.as_bytes())? {
            let session: Session = ciborium::de::from_reader(payload.as_slice())?;
            return Ok(Some(session));
        }
        Ok(None)
    }

    async fn store_session(&self, session: Session) -> async_session::Result<Option<String>> {
        let id = session.id();
        let mut payload: Vec<u8> = Vec::new();
        let cf_handle = self
            .get_column_family_handle()
            .expect(&format!("column family {} is missing", SESSION_CF));
        ciborium::ser::into_writer(&session, &mut payload)?;
        self.db
            .put_cf(&cf_handle, id.as_bytes(), payload.as_slice())?;
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> async_session::Result {
        let id = session.id();
        let cf_handle = self
            .get_column_family_handle()
            .expect(&format!("column family {} is missing", SESSION_CF));
        self.db.delete_cf(&cf_handle, id.as_bytes())?;
        Ok(())
    }

    async fn clear_store(&self) -> async_session::Result {
        self.db.drop_cf(SESSION_CF)?;
        Ok(())
    }
}
