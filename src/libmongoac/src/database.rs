use crate::error::{ErrorCodeT, ErrorT};
use crate::future::FutureT;
use crate::private::bson::bson_t;
use crate::runtime::RuntimeT;
use crate::{
    safe_as_ref_with_error, safe_cstr_from_ptr_with_error, safe_drop, safe_optional_as_ref,
    safe_optional_error_as_mut, spawn,
};

use crate::client::ClientT;
use mongodb::bson::Document;
use std::ffi::c_char;

pub struct DatabaseT {
    inner: mongodb::Database,
    runtime: RuntimeT,
}

impl DatabaseT {
    fn new(
        client: &ClientT,
        name: String,
        options: Option<Document>,
    ) -> Result<Self, mongodb::bson::error::Error> {
        let db = match options {
            Some(doc) => {
                let opts = mongodb::bson::deserialize_from_document::<
                    mongodb::options::DatabaseOptions,
                >(doc)?;
                client.client().database_with_options(&name, opts)
            }
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
    client: *mut ClientT,
    name: *const c_char,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut DatabaseT {
    let error = safe_optional_error_as_mut!(error);
    let client = match unsafe { client.as_ref() } {
        Some(c) => c,
        None => {
            if let Some(e) = error {
                *e = ErrorT::from_mongoac(ErrorCodeT::InvalidArgument, "client is null");
            }
            return Default::default();
        }
    };
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let opts_doc: Option<Document> = match safe_optional_as_ref!(options) {
        Some(bson) => match bson.to_document() {
            Ok(doc) => Some(doc),
            Err(err) => {
                if let Some(e) = error {
                    *e = ErrorT::from_bson(&err);
                }
                return Default::default();
            }
        },
        None => None,
    };

    let db = match DatabaseT::new(client, name, opts_doc) {
        Ok(db) => db,
        Err(err) => {
            if let Some(e) = error {
                *e = ErrorT::from_bson(&err);
            }
            return Default::default();
        }
    };
    Box::into_raw(Box::new(db))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_destroy(database: *mut DatabaseT) {
    safe_drop!(database);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_create_collection_async(
    database: *mut DatabaseT,
    name: *const c_char,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let create_opts: Option<mongodb::options::CreateCollectionOptions> =
        match safe_optional_as_ref!(options) {
            Some(bson) => match bson.to_document() {
                Ok(doc) => match mongodb::bson::deserialize_from_document(doc) {
                    Ok(opts) => Some(opts),
                    Err(err) => {
                        if let Some(e) = error {
                            *e = ErrorT::from_bson(&err);
                        }
                        return Default::default();
                    }
                },
                Err(err) => {
                    if let Some(e) = error {
                        *e = ErrorT::from_bson(&err);
                    }
                    return Default::default();
                }
            },
            None => None,
        };

    let future = database.create_collection_async(name, create_opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_drop_async(
    database: *mut DatabaseT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);

    let future = database.drop_async();
    Box::into_raw(Box::new(future))
}
