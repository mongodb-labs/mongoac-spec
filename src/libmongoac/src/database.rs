use crate::client_session::ClientSessionT;
use crate::create_collection_options::CreateCollectionOptionsT;
use crate::cursor::CursorT;
use crate::database_options::DatabaseOptionsT;
use crate::drop_database_options::DropDatabaseOptionsT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::list_collections_options::ListCollectionsOptionsT;
use crate::private::bson::{BsonT, bson_t};
use crate::private::macros::*;
use crate::runtime::RuntimeT;
use crate::spawn;
use crate::{cursor_op_with_session, op_with_session};

use crate::client::{ClientT, strings_to_bson};
use mongodb::Database;
use mongodb::bson::RawDocumentBuf;
use mongodb::options::{
    CreateCollectionOptions, DatabaseOptions, DropDatabaseOptions, ListCollectionsOptions,
};
use std::ffi::c_char;

pub struct DatabaseT {
    inner: Database,
    runtime: RuntimeT,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_get_database(
    client: *const ClientT,
    name: *const c_char,
    options: *const DatabaseOptionsT,
    error: *mut ErrorT,
) -> *mut DatabaseT {
    let error = safe_optional_error_as_mut!(error);
    let client = safe_as_ref_with_error!(client, error);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let options = safe_optional_as_ref!(options);

    let db = safe_error!(DatabaseT::new(client, name, options.map(Into::into)), error);
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
    options: *const DropDatabaseOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(
        database.drop_async(session, options.map(Into::into)),
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_drop(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    options: *const DropDatabaseOptionsT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    safe_error!(database.drop(session, options.map(Into::into)), error);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_create_collection_async(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    name: *const c_char,
    options: *const CreateCollectionOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let options = safe_optional_as_ref!(options);
    let create_opts = options.map(Into::into);

    let future = database.create_collection_async(name, session, create_opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_create_collection(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    name: *const c_char,
    options: *const CreateCollectionOptionsT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let options = safe_optional_as_ref!(options);
    let create_opts = options.map(Into::into);

    safe_error!(
        database.create_collection(name, session, create_opts),
        error
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_list_collections_async(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    options: *const ListCollectionsOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    let future = database.list_collections_async(session, options.map(Into::into));
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_list_collections(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    options: *const ListCollectionsOptionsT,
    error: *mut ErrorT,
) -> *mut CursorT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    let cursor = safe_error!(
        database.list_collections(session, options.map(Into::into)),
        error
    );
    Box::into_raw(Box::new(cursor))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_list_collection_names_async(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    options: *const ListCollectionsOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    let future = database.list_collection_names_async(session, options.map(Into::into));
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_list_collection_names(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    options: *const ListCollectionsOptionsT,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    let names = safe_error!(
        database.list_collection_names(session, options.map(Into::into)),
        error
    );
    safe_error!(BsonT::try_from(&names), error).into_raw()
}

impl DatabaseT {
    fn new(
        client: &ClientT,
        name: String,
        options: Option<DatabaseOptions>,
    ) -> Result<Self, mongodb::bson::error::Error> {
        let db = match options {
            Some(o) => client.inner().database_with_options(&name, o),
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
            op_with_session!(db.drop().with_options(options), session)
        })
    }

    fn drop(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<DropDatabaseOptions>,
    ) -> Result<(), ErrorT> {
        self.runtime
            .block_on(async { op_with_session!(self.inner.drop().with_options(options), session) })
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
            op_with_session!(db.create_collection(name).with_options(options), session)
        })
    }

    fn create_collection(
        &self,
        name: String,
        session: Option<&mut ClientSessionT>,
        options: Option<CreateCollectionOptions>,
    ) -> Result<(), ErrorT> {
        self.runtime.block_on(async {
            op_with_session!(
                self.inner.create_collection(name).with_options(options),
                session
            )
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
            cursor_op_with_session!(
                db.list_collections().with_options(options),
                session,
                runtime
            )
        })
    }

    fn list_collections(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListCollectionsOptions>,
    ) -> Result<CursorT, ErrorT> {
        self.runtime.block_on(async {
            cursor_op_with_session!(
                self.inner.list_collections().with_options(options),
                session,
                self.runtime.clone()
            )
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
            let res = op_with_session!(db.list_collection_names().with_options(options), session)?;
            strings_to_bson(&res)
        })
    }

    fn list_collection_names(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<ListCollectionsOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let res = op_with_session!(
                self.inner.list_collection_names().with_options(options),
                session
            )?;
            strings_to_bson(&res)
        })
    }
}
