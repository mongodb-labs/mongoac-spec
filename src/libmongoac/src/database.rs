use crate::bson::BsonT;
use crate::bson::BsonViewT;
use crate::client_session::ClientSessionT;
use crate::collection::CollectionT;
use crate::collection_options::CollectionOptionsT;
use crate::create_collection_options::CreateCollectionOptionsT;
use crate::cursor::CursorT;
use crate::drop_database_options::DropDatabaseOptionsT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::list_collections_options::ListCollectionsOptionsT;
use crate::private::macros::*;
use crate::run_command_options::RunCommandOptionsT;
use crate::run_cursor_command_options::RunCursorCommandOptionsT;
use crate::runtime::RuntimeT;
use crate::spawn;
use crate::string::StringViewT;
use crate::{cursor_op_with_session, op_with_session};

use crate::client::{ClientT, strings_to_bson};
use mongodb::Database;
use mongodb::bson::RawDocumentBuf;
use mongodb::options::{
    CreateCollectionOptions, DatabaseOptions, DropDatabaseOptions, ListCollectionsOptions,
    RunCommandOptions, RunCursorCommandOptions,
};

#[derive(Clone)]
pub struct DatabaseT {
    inner: Database,
    runtime: RuntimeT,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_destroy(database: *mut DatabaseT) {
    safe_drop!(database);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_clone(database: *const DatabaseT) -> *mut DatabaseT {
    Box::into_raw(Box::new(safe_as_ref!(database).clone()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_get_collection(
    database: *const DatabaseT,
    name: StringViewT,
    options: *const CollectionOptionsT,
    error: *mut ErrorT,
) -> *mut CollectionT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let name = safe_string_view_with_error!(name, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(CollectionT::new(
        database,
        name,
        options.map(Into::into),
    )))
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
    name: StringViewT,
    options: *const CreateCollectionOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let name = safe_string_view_with_error!(name, error);
    let options = safe_optional_as_ref!(options);
    let create_opts = options.map(Into::into);

    let future = database.create_collection_async(name, session, create_opts);
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_create_collection(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    name: StringViewT,
    options: *const CreateCollectionOptionsT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let name = safe_string_view_with_error!(name, error);
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
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    safe_error!(
        database.list_collection_names(session, options.map(Into::into)),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_run_command_async(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    command: BsonViewT,
    options: *const RunCommandOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let command = safe_bson_view_with_error!(command, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(database.run_command_async(
        session,
        safe_error!((&command).try_into(), error), // Deep-copy!
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_run_command(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    command: BsonViewT,
    options: *const RunCommandOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let command = safe_bson_view_with_error!(command, error);
    let options = safe_optional_as_ref!(options);

    safe_error!(
        database.run_command(
            session,
            safe_error!((&command).try_into(), error), // Deep-copy!
            options.map(Into::into)
        ),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_run_cursor_command_async(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    command: BsonViewT,
    options: *const RunCursorCommandOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let command = safe_bson_view_with_error!(command, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(database.run_cursor_command_async(
        session,
        safe_error!((&command).try_into(), error), // Deep-copy!
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_run_cursor_command(
    database: *const DatabaseT,
    session: *mut ClientSessionT,
    command: BsonViewT,
    options: *const RunCursorCommandOptionsT,
    error: *mut ErrorT,
) -> *mut CursorT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let session = safe_optional_as_mut!(session);
    let command = safe_bson_view_with_error!(command, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(safe_error!(
        database.run_cursor_command(
            session,
            safe_error!((&command).try_into(), error), // Deep-copy!
            options.map(Into::into)
        ),
        error
    )))
}

impl DatabaseT {
    #[must_use]
    pub fn new(client: &ClientT, name: &str, options: Option<DatabaseOptions>) -> Self {
        let db = match options {
            Some(o) => client.inner().database_with_options(name, o),
            None => client.inner().database(name),
        };

        DatabaseT {
            inner: db,
            runtime: client.get_runtime(),
        }
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
        name: &str,
        session: Option<&mut ClientSessionT>,
        options: Option<CreateCollectionOptions>,
    ) -> FutureT {
        let db = self.inner.clone();
        let name = name.to_string();
        let session = session.map(|s| s.clone());

        spawn!(self, Void, async move {
            op_with_session!(db.create_collection(name).with_options(options), session)
        })
    }

    fn create_collection(
        &self,
        name: &str,
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

    fn run_command_async(
        &self,
        session: Option<&mut ClientSessionT>,
        command: RawDocumentBuf,
        options: Option<RunCommandOptions>,
    ) -> FutureT {
        let db = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Bson, async move {
            let doc = op_with_session!(db.run_raw_command(command).with_options(options), session)?;
            RawDocumentBuf::try_from(&doc).map_err(Into::into)
        })
    }

    fn run_command(
        &self,
        session: Option<&mut ClientSessionT>,
        command: RawDocumentBuf,
        options: Option<RunCommandOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let doc = op_with_session!(
                self.inner.run_raw_command(command).with_options(options),
                session
            )?;
            RawDocumentBuf::try_from(&doc).map_err(Into::into)
        })
    }

    fn run_cursor_command_async(
        &self,
        session: Option<&mut ClientSessionT>,
        command: RawDocumentBuf,
        options: Option<RunCursorCommandOptions>,
    ) -> FutureT {
        let db = self.inner.clone();
        let session = session.map(|s| s.clone());
        let runtime = self.runtime.clone();

        spawn!(self, Cursor, async move {
            cursor_op_with_session!(
                db.run_raw_cursor_command(command).with_options(options),
                session,
                runtime
            )
        })
    }

    fn run_cursor_command(
        &self,
        session: Option<&mut ClientSessionT>,
        command: RawDocumentBuf,
        options: Option<RunCursorCommandOptions>,
    ) -> Result<CursorT, ErrorT> {
        self.runtime.block_on(async {
            cursor_op_with_session!(
                self.inner
                    .run_raw_cursor_command(command)
                    .with_options(options),
                session,
                self.runtime.clone()
            )
        })
    }
}
