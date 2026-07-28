use crate::client_session::ClientSessionT;
use crate::database::DatabaseT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::bson::bson_t;
use crate::private::drop_collection_options::DropCollectionOptionsT;
use crate::private::macros::*;
use crate::runtime::RuntimeT;
use crate::spawn;

use mongodb::Collection;
use mongodb::bson::RawDocumentBuf;
use mongodb::options::DropCollectionOptions;

use std::ffi::c_char;

pub struct CollectionT {
    inner: Collection<RawDocumentBuf>,
    runtime: RuntimeT,
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

    Box::into_raw(Box::new(CollectionT::new(database, name)))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_destroy(collection: *mut CollectionT) {
    safe_drop!(collection);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_drop_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_bson_opts_with_error!(DropCollectionOptionsT, options, error);

    Box::into_raw(Box::new(
        collection.drop_async(session, options.map(Into::into)),
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_drop(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_bson_opts_with_error!(DropCollectionOptionsT, options, error);

    safe_error!(collection.drop(session, options.map(Into::into)), error);
}

impl CollectionT {
    fn new(db: &DatabaseT, name: String) -> Self {
        let coll = db.inner().collection::<RawDocumentBuf>(&name);

        CollectionT {
            inner: coll,
            runtime: db.get_runtime(),
        }
    }

    fn drop_async(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<DropCollectionOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Void, async move {
            let op = coll.drop().with_options(options);

            match session {
                Some(session) => {
                    let mut guard = session.state().lock().await;
                    op.session(&mut *guard).await?
                }
                None => op.await?,
            }

            Ok::<(), ErrorT>(())
        })
    }

    fn drop(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<DropCollectionOptions>,
    ) -> Result<(), ErrorT> {
        self.runtime.block_on(async {
            let op = self.inner.drop().with_options(options);

            match session {
                Some(session) => {
                    let mut guard = session.state().lock().await;
                    op.session(&mut *guard).await?
                }
                None => op.await?,
            }

            Ok::<(), ErrorT>(())
        })
    }
}
