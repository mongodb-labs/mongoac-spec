use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::bson::bson_t;
use crate::private::macros::*;
use crate::runtime::RuntimeT;
use crate::spawn;

use crate::client::ClientT;
use mongodb::options::{CreateCollectionOptions, DatabaseOptions};
use std::ffi::c_char;

pub struct DatabaseT {
    inner: mongodb::Database,
    runtime: RuntimeT,
}

impl DatabaseT {
    fn new(
        client: &ClientT,
        name: String,
        options: Option<DatabaseOptions>,
    ) -> Result<Self, mongodb::bson::error::Error> {
        let db = match options {
            Some(opts) => client.client().database_with_options(&name, opts),
            None => client.client().database(&name),
        };

        Ok(DatabaseT {
            inner: db,
            runtime: client.get_runtime(),
        })
    }

    pub(crate) fn database(&self) -> &mongodb::Database {
        &self.inner
    }

    pub(crate) fn get_runtime(&self) -> RuntimeT {
        self.runtime.clone()
    }

    fn create_collection_async(
        &self,
        name: String,
        options: Option<mongodb::options::CreateCollectionOptions>,
    ) -> FutureT {
        let db = self.inner.clone();
        spawn!(self, Void, async move {
            let mut action = db.create_collection(name);
            if let Some(opts) = options {
                action = action.with_options(opts);
            }
            action.await
        })
    }

    fn drop_async(&self) -> FutureT {
        let db = self.inner.clone();
        spawn!(self, Void, async move { db.drop().await })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_get_database(
    client: *const ClientT,
    name: *const c_char,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut DatabaseT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let opts = safe_optional_bson_options!(options, error, DatabaseOptions);

    let db = safe_error!(DatabaseT::new(client, name, opts), error);
    Box::into_raw(Box::new(db))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_destroy(database: *mut DatabaseT) {
    safe_drop!(database);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_create_collection_async(
    database: *const DatabaseT,
    name: *const c_char,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let create_opts = safe_optional_bson_options!(options, error, CreateCollectionOptions);

    let future = database.create_collection_async(name, create_opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_drop_async(
    database: *const DatabaseT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);

    let future = database.drop_async();
    Box::into_raw(Box::new(future))
}
