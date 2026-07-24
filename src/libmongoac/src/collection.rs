use std::ffi::c_char;

use crate::database::DatabaseT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::bson::bson_t;
use crate::private::macros::*;
use crate::runtime::RuntimeT;
use crate::spawn;
use mongodb::bson::RawDocumentBuf;

pub struct CollectionT {
    inner: mongodb::Collection<RawDocumentBuf>,
    runtime: RuntimeT,
}

impl CollectionT {
    fn new(db: &DatabaseT, name: String) -> Self {
        let coll = db.database().collection::<RawDocumentBuf>(&name);

        CollectionT {
            inner: coll,
            runtime: db.get_runtime(),
        }
    }

    fn drop_async(&self) -> FutureT {
        let coll = self.inner.clone();
        spawn!(self, Void, async move { coll.drop().await })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_get_collection(
    database: *const DatabaseT,
    name: *const c_char,
    error: *mut ErrorT,
) -> *mut CollectionT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let coll = CollectionT::new(database, name);
    Box::into_raw(Box::new(coll))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_destroy(collection: *mut CollectionT) {
    safe_drop!(collection);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_drop_async(
    collection: *const CollectionT,
    _options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);

    let future = collection.drop_async();
    Box::into_raw(Box::new(future))
}
