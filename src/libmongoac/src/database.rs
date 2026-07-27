use crate::client_session::ClientSessionT;
use crate::cursor::CursorT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::bson::{BsonT, bson_t};
use crate::private::macros::*;
use crate::runtime::RuntimeT;
use crate::spawn;

use crate::client::{ClientT, strings_to_bson};
use mongodb::Database;
use mongodb::bson::RawDocumentBuf;
use mongodb::options::{
    CreateCollectionOptions, DatabaseOptions, DropDatabaseOptions, ListCollectionsOptions,
    ReadConcern, ReadPreference, SelectionCriteria, WriteConcern,
};
use serde::Deserialize;
use std::ffi::c_char;

pub struct DatabaseT {
    inner: Database,
    runtime: RuntimeT,
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
    let opts = safe_optional_bson_opts_with_error!(DatabaseOptionsT, options, error);

    let db = safe_error!(DatabaseT::new(client, name, opts), error);
    Box::into_raw(Box::new(db))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_destroy(database: *mut DatabaseT) {
    safe_drop!(database);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_drop_async(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    _options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);

    // let options = safe_optional_bson_opts_with_error!(DropDatabaseOptions, options, error);
    // - write_concern: serde(skip_serializing)
    let options = None;

    Box::into_raw(Box::new(database.drop_async(session, options)))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_drop(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    _options: *const bson_t,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);

    // let options = safe_optional_bson_opts_with_error!(DropDatabaseOptions, options, error);
    // - write_concern: serde(skip_serializing)
    let options = None;

    safe_error!(database.drop(session, options), error);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_create_collection_async(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    name: *const c_char,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let create_opts = safe_optional_bson_opts_with_error!(CreateCollectionOptions, options, error);

    let future = database.create_collection_async(name, session, create_opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_create_collection(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    name: *const c_char,
    options: *const bson_t,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let create_opts = safe_optional_bson_opts_with_error!(CreateCollectionOptions, options, error);

    safe_error!(
        database.create_collection(name, session, create_opts),
        error
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_list_collections_async(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let opts = safe_optional_bson_opts_with_error!(ListCollectionsOptions, options, error);

    let future = database.list_collections_async(session, opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_list_collections(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut CursorT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let opts = safe_optional_bson_opts_with_error!(ListCollectionsOptions, options, error);

    let cursor = safe_error!(database.list_collections(session, opts), error);
    Box::into_raw(Box::new(cursor))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_list_collection_names_async(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let opts = safe_optional_bson_opts_with_error!(ListCollectionsOptions, options, error);

    let future = database.list_collection_names_async(session, opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_list_collection_names(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let opts = safe_optional_bson_opts_with_error!(ListCollectionsOptions, options, error);

    let names = safe_error!(database.list_collection_names(session, opts), error);
    safe_error!(BsonT::try_from(&names), error).into_raw()
}

impl DatabaseT {
    fn new(
        client: &ClientT,
        name: String,
        options: Option<DatabaseOptionsT>,
    ) -> Result<Self, mongodb::bson::error::Error> {
        let db = match options {
            Some(opts) => client.inner().database_with_options(&name, opts.into()),
            None => client.inner().database(&name),
        };

        Ok(DatabaseT {
            inner: db,
            runtime: client.get_runtime(),
        })
    }

    pub(crate) fn inner(&self) -> &Database {
        &self.inner
    }

    pub(crate) fn get_runtime(&self) -> RuntimeT {
        self.runtime.clone()
    }

    fn drop_async(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<DropDatabaseOptions>,
    ) -> FutureT {
        let db = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Void, async move {
            let op = db.drop().with_options(options);

            match session {
                Some(state) => {
                    let mut guard = state.state().lock().await;
                    op.session(&mut *guard).await?
                }
                None => op.await?,
            };

            Ok::<(), ErrorT>(())
        })
    }

    fn drop(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<DropDatabaseOptions>,
    ) -> Result<(), ErrorT> {
        self.runtime.block_on(async {
            let op = self.inner.drop().with_options(options);

            match session {
                Some(state) => {
                    let mut guard = state.state().lock().await;
                    op.session(&mut *guard).await?
                }
                None => op.await?,
            };

            Ok::<(), ErrorT>(())
        })
    }

    fn create_collection_async(
        &self,
        name: String,
        session: Option<&mut ClientSessionT>,
        options: Option<CreateCollectionOptions>,
    ) -> FutureT {
        let db = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Void, async move {
            let op = db.create_collection(name).with_options(options);

            match session {
                Some(state) => {
                    let mut guard = state.state().lock().await;
                    op.session(&mut *guard).await?
                }
                None => op.await?,
            };

            Ok::<(), ErrorT>(())
        })
    }

    fn create_collection(
        &self,
        name: String,
        session: Option<&mut ClientSessionT>,
        options: Option<CreateCollectionOptions>,
    ) -> Result<(), ErrorT> {
        self.runtime.block_on(async {
            let op = self.inner.create_collection(name).with_options(options);

            match session {
                Some(state) => {
                    let mut guard = state.state().lock().await;
                    op.session(&mut *guard).await?
                }
                None => op.await?,
            };

            Ok::<(), ErrorT>(())
        })
    }

    fn list_collections_async(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListCollectionsOptions>,
    ) -> FutureT {
        let db = self.inner.clone();
        let session = session.map(|s| s.clone());
        let runtime = self.runtime.clone();

        spawn!(&self, Cursor, async move {
            let op = db.list_collections().with_options(options);

            let res = match session {
                Some(session) => {
                    let mut guard = session.state().lock().await;
                    CursorT::new_with_session(
                        op.session(&mut *guard).await?.with_type(),
                        session.clone(),
                        runtime,
                    )
                }
                None => CursorT::new(op.await?.with_type(), runtime),
            };

            Ok::<CursorT, ErrorT>(res)
        })
    }

    fn list_collections(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListCollectionsOptions>,
    ) -> Result<CursorT, ErrorT> {
        self.runtime.block_on(async {
            let op = self.inner.list_collections().with_options(options);

            let res = match session {
                Some(session) => {
                    let mut guard = session.state().lock().await;
                    CursorT::new_with_session(
                        op.session(&mut *guard).await?.with_type(),
                        session.clone(),
                        self.runtime.clone(),
                    )
                }
                None => CursorT::new(op.await?.with_type(), self.runtime.clone()),
            };

            Ok::<CursorT, ErrorT>(res)
        })
    }

    fn list_collection_names_async(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListCollectionsOptions>,
    ) -> FutureT {
        let db = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(&self, Bson, async move {
            let op = db.list_collection_names().with_options(options);

            let res = match session {
                Some(session) => {
                    let mut guard = session.state().lock().await;
                    op.session(&mut *guard).await?
                }
                None => op.await?,
            };

            strings_to_bson(&res)
        })
    }

    fn list_collection_names(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListCollectionsOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let op = self.inner.list_collection_names().with_options(options);

            let res = match session {
                Some(session) => {
                    let mut guard = session.state().lock().await;
                    op.session(&mut *guard).await?
                }
                None => op.await?,
            };

            strings_to_bson(&res)
        })
    }
}

#[derive(Deserialize)]
struct DatabaseOptionsT {
    #[serde(alias = "readConcern")]
    read_concern: Option<ReadConcern>,

    #[serde(alias = "readPreference")]
    read_preference: Option<ReadPreference>,

    #[serde(alias = "writeConcern")]
    write_concern: Option<WriteConcern>,
}

impl From<DatabaseOptionsT> for DatabaseOptions {
    fn from(opts: DatabaseOptionsT) -> Self {
        DatabaseOptions::builder()
            .read_concern(opts.read_concern)
            .selection_criteria(
                opts.read_preference
                    .map(SelectionCriteria::ReadPreference),
            )
            .write_concern(opts.write_concern)
            .build()
    }
}
