use crate::client_session::ClientSessionT;
use crate::cursor::CursorT;
use crate::database::DatabaseT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::bson::{BsonT, bson_t};
use crate::private::drop_collection_options::DropCollectionOptionsT;
use crate::private::macros::*;
use crate::runtime::RuntimeT;
use crate::spawn;
use crate::{cursor_op_with_session, op_with_session};

use mongodb::Collection;
use mongodb::bson::{Document, RawDocument, RawDocumentBuf, serialize_to_raw_document_buf};
use mongodb::options::{DropCollectionOptions, FindOptions, InsertManyOptions, InsertOneOptions};
use mongodb::results::InsertManyResult;

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

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_find_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: *const bson_t,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_const_bson_with_error!(filter, error);
    let options = safe_optional_bson_opts_with_error!(FindOptions, options, error);

    Box::into_raw(Box::new(collection.find_async(
        session,
        safe_error!((&filter).try_into(), error),
        options,
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_find(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: *const bson_t,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut CursorT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_const_bson_with_error!(filter, error);
    let options = safe_optional_bson_opts_with_error!(FindOptions, options, error);

    Box::into_raw(Box::new(safe_error!(
        collection.find(session, safe_error!((&filter).try_into(), error), options),
        error
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_insert_one_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    document: *const bson_t,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let document = safe_const_bson_with_error!(document, error);
    let options =
        safe_optional_bson_opts_with_error!(mongodb::options::InsertOneOptions, options, error);

    Box::into_raw(Box::new(collection.insert_one_async(
        session,
        safe_error!((&document).try_into(), error),
        options,
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_insert_one(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    document: *const bson_t,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let document = safe_const_bson_with_error!(document, error);
    let options =
        safe_optional_bson_opts_with_error!(mongodb::options::InsertOneOptions, options, error);

    let result = safe_error!(
        collection.insert_one(session, safe_error!((&document).try_into(), error), options),
        error
    );

    safe_error!(BsonT::try_from(&result), error).into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_insert_many_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    documents: *const *const bson_t,
    count: usize,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let documents = safe_const_bson_array_as_vec_with_error!(documents, count, error);
    let documents: Vec<&RawDocument> = safe_error!(
        documents
            .iter()
            .map(|d| d.try_into())
            .collect::<Result<_, _>>(),
        error
    );
    let options = safe_optional_bson_opts_with_error!(InsertManyOptions, options, error);

    Box::into_raw(Box::new(
        collection.insert_many_async(session, documents, options),
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_insert_many(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    documents: *const *const bson_t,
    count: usize,
    options: *const bson_t,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let documents = safe_const_bson_array_as_vec_with_error!(documents, count, error);
    let documents: Vec<&RawDocument> = safe_error!(
        documents
            .iter()
            .map(|d| d.try_into())
            .collect::<Result<_, _>>(),
        error
    );
    let options = safe_optional_bson_opts_with_error!(InsertManyOptions, options, error);

    let result = safe_error!(collection.insert_many(session, documents, options), error);

    safe_error!(BsonT::try_from(&result), error).into_raw()
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
            op_with_session!(coll.drop().with_options(options), session)
        })
    }

    fn drop(
        &self,
        session: Option<&mut ClientSessionT>,
        options: Option<DropCollectionOptions>,
    ) -> Result<(), ErrorT> {
        self.runtime
            .block_on(async { op_with_session!(self.inner.drop().with_options(options), session) })
    }

    fn find_async(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<mongodb::options::FindOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());
        let runtime = self.runtime.clone();

        spawn!(self, Cursor, async move {
            cursor_op_with_session!(coll.find(filter).with_options(options), session, runtime)
        })
    }

    fn find(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<mongodb::options::FindOptions>,
    ) -> Result<CursorT, ErrorT> {
        self.runtime.block_on(async {
            cursor_op_with_session!(
                self.inner.find(filter).with_options(options),
                session,
                self.runtime.clone()
            )
        })
    }

    fn insert_one_async(
        &self,
        session: Option<&mut ClientSessionT>,
        document: &RawDocument,
        options: Option<InsertOneOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());
        let document = document.to_owned(); // Deep-copy!

        spawn!(self, Bson, async move {
            let res = op_with_session!(coll.insert_one(document).with_options(options), session)?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn insert_one(
        &self,
        session: Option<&mut ClientSessionT>,
        document: &RawDocument,
        options: Option<InsertOneOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        let coll = self.inner.clone_with_type::<&RawDocument>();

        self.runtime.block_on(async {
            let res = op_with_session!(coll.insert_one(document).with_options(options), session)?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn insert_many_async(
        &self,
        session: Option<&mut ClientSessionT>,
        documents: Vec<&RawDocument>,
        options: Option<InsertManyOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());
        let documents: Vec<RawDocumentBuf> = documents.into_iter().map(|d| d.to_owned()).collect(); // Deep-copy!

        spawn!(self, Bson, async move {
            let res = op_with_session!(coll.insert_many(documents).with_options(options), session)?;
            insert_many_result_to_raw(&res)
        })
    }

    fn insert_many(
        &self,
        session: Option<&mut ClientSessionT>,
        documents: Vec<&RawDocument>,
        options: Option<InsertManyOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        let coll = self.inner.clone_with_type::<&RawDocument>();

        self.runtime.block_on(async {
            let res = op_with_session!(coll.insert_many(documents).with_options(options), session)?;
            insert_many_result_to_raw(&res)
        })
    }
}

fn insert_many_result_to_raw(res: &InsertManyResult) -> Result<RawDocumentBuf, ErrorT> {
    let mut inserted_ids = Document::new();
    for (index, id) in &res.inserted_ids {
        // Cannot serialize keys as integers: `{0: ...}` (invalid) vs. `{"0": ...}` (valid).
        inserted_ids.insert(index.to_string(), id.clone());
    }
    let mut doc = Document::new();
    doc.insert("insertedIds", inserted_ids);
    serialize_to_raw_document_buf(&doc).map_err(Into::into)
}
