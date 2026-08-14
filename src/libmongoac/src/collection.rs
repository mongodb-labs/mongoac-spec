use crate::aggregate_options::AggregateOptionsT;
use crate::bson::{BsonT, BsonViewT};
use crate::client_session::ClientSessionT;
use crate::collection_options::CollectionOptionsT;
use crate::count_options::CountOptionsT;
use crate::cursor::CursorT;
use crate::database::DatabaseT;
use crate::delete_options::DeleteOptionsT;
use crate::distinct_options::DistinctOptionsT;
use crate::drop_collection_options::DropCollectionOptionsT;
use crate::error::ErrorT;
use crate::estimated_document_count_options::EstimatedDocumentCountOptionsT;
use crate::find_one_options::FindOneOptionsT;
use crate::find_options::FindOptionsT;
use crate::future::FutureT;
use crate::insert_many_options::InsertManyOptionsT;
use crate::insert_one_options::InsertOneOptionsT;
use crate::private::macros::*;
use crate::replace_options::ReplaceOptionsT;
use crate::runtime::RuntimeT;
use crate::spawn;
use crate::string::StringViewT;
use crate::update_options::UpdateOptionsT;
use crate::{cursor_op_with_session, op_with_session};

use mongodb::Collection;
use mongodb::bson::{Document, RawDocument, RawDocumentBuf, serialize_to_raw_document_buf};
use mongodb::options::{
    AggregateOptions, CountOptions, DeleteOptions, DistinctOptions, DropCollectionOptions,
    EstimatedDocumentCountOptions, FindOneOptions, FindOptions, InsertManyOptions,
    InsertOneOptions, ReplaceOptions, UpdateOptions,
};
use mongodb::results::InsertManyResult;

pub struct CollectionT {
    inner: Collection<RawDocumentBuf>,
    runtime: RuntimeT,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_get_collection(
    database: *const DatabaseT,
    name: StringViewT,
    error: *mut ErrorT,
) -> *mut CollectionT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let name = safe_string_view_with_error!(name, error);

    Box::into_raw(Box::new(CollectionT::new(database, name)))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_get_collection_with_options(
    database: *const DatabaseT,
    name: StringViewT,
    options: *const CollectionOptionsT,
    error: *mut ErrorT,
) -> *mut CollectionT {
    let error = safe_optional_error_as_mut!(error);
    let database = safe_as_ref_with_error!(database, error);
    let name = safe_string_view_with_error!(name, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(CollectionT::new_with_options(
        database, name, options,
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_destroy(collection: *mut CollectionT) {
    safe_drop!(collection);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_drop_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    options: *const DropCollectionOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(
        collection.drop_async(session, options.map(Into::into)),
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_drop(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    options: *const DropCollectionOptionsT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let options = safe_optional_as_ref!(options);

    safe_error!(collection.drop(session, options.map(Into::into)), error);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_find_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const FindOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.find_async(
        session,
        safe_error!((&filter).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_find(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const FindOptionsT,
    error: *mut ErrorT,
) -> *mut CursorT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(safe_error!(
        collection.find(
            session,
            safe_error!((&filter).try_into(), error),
            options.map(Into::into)
        ),
        error
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_insert_one_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    document: BsonViewT,
    options: *const InsertOneOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let document = safe_bson_view_with_error!(document, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.insert_one_async(
        session,
        safe_error!((&document).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_insert_one(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    document: BsonViewT,
    options: *const InsertOneOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let document = safe_bson_view_with_error!(document, error);
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.insert_one(
            session,
            safe_error!((&document).try_into(), error),
            options.map(Into::into)
        ),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_insert_many_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    documents: *const BsonViewT,
    count: usize,
    options: *const InsertManyOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let documents = safe_bson_view_array_as_vec_with_error!(documents, count, error);
    let documents: Vec<&RawDocument> = safe_error!(
        documents
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>(),
        error
    );
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.insert_many_async(
        session,
        documents,
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_insert_many(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    documents: *const BsonViewT,
    count: usize,
    options: *const InsertManyOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let documents = safe_bson_view_array_as_vec_with_error!(documents, count, error);
    let documents: Vec<&RawDocument> = safe_error!(
        documents
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>(),
        error
    );
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.insert_many(session, documents, options.map(Into::into)),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_delete_one_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const DeleteOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.delete_one_async(
        session,
        safe_error!((&filter).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_delete_one(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const DeleteOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.delete_one(
            session,
            safe_error!((&filter).try_into(), error),
            options.map(Into::into)
        ),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_delete_many_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const DeleteOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.delete_many_async(
        session,
        safe_error!((&filter).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_delete_many(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const DeleteOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.delete_many(
            session,
            safe_error!((&filter).try_into(), error),
            options.map(Into::into)
        ),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_replace_one_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    replacement: BsonViewT,
    options: *const ReplaceOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let replacement = safe_bson_view_with_error!(replacement, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.replace_one_async(
        session,
        safe_error!((&filter).try_into(), error),
        safe_error!((&replacement).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_replace_one(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    replacement: BsonViewT,
    options: *const ReplaceOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let replacement = safe_bson_view_with_error!(replacement, error);
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.replace_one(
            session,
            safe_error!((&filter).try_into(), error),
            safe_error!((&replacement).try_into(), error),
            options.map(Into::into)
        ),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_update_one_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    update: BsonViewT,
    options: *const UpdateOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let update = safe_bson_view_with_error!(update, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.update_one_async(
        session,
        safe_error!((&filter).try_into(), error),
        safe_error!((&update).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_update_one(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    update: BsonViewT,
    options: *const UpdateOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let update = safe_bson_view_with_error!(update, error);
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.update_one(
            session,
            safe_error!((&filter).try_into(), error),
            safe_error!((&update).try_into(), error),
            options.map(Into::into)
        ),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_update_many_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    update: BsonViewT,
    options: *const UpdateOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let update = safe_bson_view_with_error!(update, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.update_many_async(
        session,
        safe_error!((&filter).try_into(), error),
        safe_error!((&update).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_update_many(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    update: BsonViewT,
    options: *const UpdateOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let update = safe_bson_view_with_error!(update, error);
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.update_many(
            session,
            safe_error!((&filter).try_into(), error),
            safe_error!((&update).try_into(), error),
            options.map(Into::into)
        ),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_count_documents_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const CountOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.count_documents_async(
        session,
        safe_error!((&filter).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_count_documents(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const CountOptionsT,
    error: *mut ErrorT,
) -> u64 {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.count_documents(
            session,
            safe_error!((&filter).try_into(), error),
            options.map(Into::into)
        ),
        error
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_estimated_document_count_async(
    collection: *const CollectionT,
    options: *const EstimatedDocumentCountOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(
        collection.estimated_document_count_async(options.map(Into::into)),
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_estimated_document_count(
    collection: *const CollectionT,
    options: *const EstimatedDocumentCountOptionsT,
    error: *mut ErrorT,
) -> u64 {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.estimated_document_count(options.map(Into::into)),
        error
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_distinct_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    field_name: StringViewT,
    filter: BsonViewT,
    options: *const DistinctOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let field_name = safe_string_view_with_error!(field_name, error);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.distinct_async(
        session,
        field_name,
        safe_error!((&filter).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_distinct(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    field_name: StringViewT,
    filter: BsonViewT,
    options: *const DistinctOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let field_name = safe_string_view_with_error!(field_name, error);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.distinct(
            session,
            field_name,
            safe_error!((&filter).try_into(), error),
            options.map(Into::into)
        ),
        error
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_aggregate_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    pipeline: *const BsonViewT,
    count: usize,
    options: *const AggregateOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let pipeline = safe_bson_view_array_as_vec_with_error!(pipeline, count, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.aggregate_async(
        session,
        safe_error!(pipeline.iter().map(TryInto::try_into).collect(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_aggregate(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    pipeline: *const BsonViewT,
    count: usize,
    options: *const AggregateOptionsT,
    error: *mut ErrorT,
) -> *mut CursorT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let pipeline = safe_bson_view_array_as_vec_with_error!(pipeline, count, error);
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(safe_error!(
        collection.aggregate(
            session,
            safe_error!(pipeline.iter().map(TryInto::try_into).collect(), error),
            options.map(Into::into)
        ),
        error
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_find_one_async(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const FindOneOptionsT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    Box::into_raw(Box::new(collection.find_one_async(
        session,
        safe_error!((&filter).try_into(), error),
        options.map(Into::into),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_find_one(
    collection: *const CollectionT,
    session: *mut ClientSessionT,
    filter: BsonViewT,
    options: *const FindOneOptionsT,
    error: *mut ErrorT,
) -> BsonT {
    let error = safe_optional_error_as_mut!(error);
    let collection = safe_as_ref_with_error!(collection, error);
    let session = safe_optional_as_mut!(session);
    let filter = safe_optional_bson_view!(filter).unwrap_or(BsonViewT::empty_doc());
    let options = safe_optional_as_ref!(options);

    safe_error!(
        collection.find_one(
            session,
            safe_error!((&filter).try_into(), error),
            options.map(Into::into)
        ),
        error
    )
    .map(Into::into)
    .unwrap_or_default()
}

impl CollectionT {
    fn new(db: &DatabaseT, name: &str) -> Self {
        let coll = db.inner().collection::<RawDocumentBuf>(name);

        CollectionT {
            inner: coll,
            runtime: db.get_runtime(),
        }
    }

    fn new_with_options(db: &DatabaseT, name: &str, options: Option<&CollectionOptionsT>) -> Self {
        let coll = match options {
            Some(opts) => db
                .inner()
                .collection_with_options::<RawDocumentBuf>(name, opts.into()),
            None => db.inner().collection::<RawDocumentBuf>(name),
        };

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
        options: Option<FindOptions>,
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
        options: Option<FindOptions>,
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
        let documents: Vec<RawDocumentBuf> = documents.into_iter().map(ToOwned::to_owned).collect(); // Deep-copy!

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

    fn delete_one_async(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<DeleteOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Bson, async move {
            let res = op_with_session!(coll.delete_one(filter).with_options(options), session)?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn delete_one(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<DeleteOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let res =
                op_with_session!(self.inner.delete_one(filter).with_options(options), session)?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn delete_many_async(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<DeleteOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Bson, async move {
            let res = op_with_session!(coll.delete_many(filter).with_options(options), session)?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn delete_many(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<DeleteOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let res = op_with_session!(
                self.inner.delete_many(filter).with_options(options),
                session
            )?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn replace_one_async(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        replacement: RawDocumentBuf,
        options: Option<ReplaceOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Bson, async move {
            let res = op_with_session!(
                coll.replace_one(filter, replacement).with_options(options),
                session
            )?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn replace_one(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        replacement: RawDocumentBuf,
        options: Option<ReplaceOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let res = op_with_session!(
                self.inner
                    .replace_one(filter, replacement)
                    .with_options(options),
                session
            )?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn update_one_async(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Bson, async move {
            let res = op_with_session!(
                coll.update_one(filter, update).with_options(options),
                session
            )?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn update_one(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let res = op_with_session!(
                self.inner.update_one(filter, update).with_options(options),
                session
            )?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn update_many_async(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, Bson, async move {
            let res = op_with_session!(
                coll.update_many(filter, update).with_options(options),
                session
            )?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn update_many(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let res = op_with_session!(
                self.inner.update_many(filter, update).with_options(options),
                session
            )?;
            serialize_to_raw_document_buf(&res).map_err(Into::into)
        })
    }

    fn count_documents_async(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<CountOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, UInt64, async move {
            op_with_session!(coll.count_documents(filter).with_options(options), session)
        })
    }

    fn count_documents(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<CountOptions>,
    ) -> Result<u64, ErrorT> {
        self.runtime.block_on(async {
            op_with_session!(
                self.inner.count_documents(filter).with_options(options),
                session
            )
        })
    }

    fn estimated_document_count_async(
        &self,
        options: Option<EstimatedDocumentCountOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();

        spawn!(self, UInt64, async move {
            coll.estimated_document_count()
                .with_options(options)
                .await
                .map_err(ErrorT::from)
        })
    }

    fn estimated_document_count(
        &self,
        options: Option<EstimatedDocumentCountOptions>,
    ) -> Result<u64, ErrorT> {
        self.runtime.block_on(async {
            self.inner
                .estimated_document_count()
                .with_options(options)
                .await
                .map_err(ErrorT::from)
        })
    }

    fn distinct_async(
        &self,
        session: Option<&mut ClientSessionT>,
        field_name: &str,
        filter: Document,
        options: Option<DistinctOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());
        let field_name = field_name.to_string();

        spawn!(self, Bson, async move {
            let values = op_with_session!(
                coll.distinct(field_name, filter).with_options(options),
                session
            )?;

            let mut doc = Document::new();
            doc.insert("v", values);
            RawDocumentBuf::try_from(doc).map_err(Into::into)
        })
    }

    fn distinct(
        &self,
        session: Option<&mut ClientSessionT>,
        field_name: &str,
        filter: Document,
        options: Option<DistinctOptions>,
    ) -> Result<RawDocumentBuf, ErrorT> {
        self.runtime.block_on(async {
            let values = op_with_session!(
                self.inner
                    .distinct(field_name, filter)
                    .with_options(options),
                session
            )?;

            let mut res = Document::new();
            res.insert("v", values);
            RawDocumentBuf::try_from(res).map_err(Into::into)
        })
    }

    fn aggregate_async(
        &self,
        session: Option<&mut ClientSessionT>,
        pipeline: Vec<Document>,
        options: Option<AggregateOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());
        let runtime = self.runtime.clone();

        spawn!(self, Cursor, async move {
            cursor_op_with_session!(
                coll.aggregate(pipeline).with_options(options),
                session,
                runtime
            )
        })
    }

    fn aggregate(
        &self,
        session: Option<&mut ClientSessionT>,
        pipeline: Vec<Document>,
        options: Option<AggregateOptions>,
    ) -> Result<CursorT, ErrorT> {
        self.runtime.block_on(async {
            cursor_op_with_session!(
                self.inner.aggregate(pipeline).with_options(options),
                session,
                self.runtime.clone()
            )
        })
    }

    fn find_one_async(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<FindOneOptions>,
    ) -> FutureT {
        let coll = self.inner.clone();
        let session = session.map(|s| s.clone());

        spawn!(self, OptionalBson, async move {
            op_with_session!(coll.find_one(filter).with_options(options), session)
        })
    }

    fn find_one(
        &self,
        session: Option<&mut ClientSessionT>,
        filter: Document,
        options: Option<FindOneOptions>,
    ) -> Result<Option<RawDocumentBuf>, ErrorT> {
        self.runtime.block_on(async {
            op_with_session!(self.inner.find_one(filter).with_options(options), session)
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
