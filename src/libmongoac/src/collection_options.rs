use crate::private::macros::*;
use crate::read_concern::ReadConcernT;
use crate::read_preference::ReadPreferenceT;
use crate::server_selector::ServerSelectorT;
use crate::write_concern::WriteConcernT;

use mongodb::options::CollectionOptions;

pub struct CollectionOptionsT(CollectionOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_options_new() -> *mut CollectionOptionsT {
    Box::into_raw(Box::new(CollectionOptionsT(CollectionOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_options_destroy(opts: *mut CollectionOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_options_set_read_concern(
    opts: *mut CollectionOptionsT,
    v: *const ReadConcernT,
) {
    safe_as_mut!(opts).0.read_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_options_set_write_concern(
    opts: *mut CollectionOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_options_set_read_preference(
    opts: *mut CollectionOptionsT,
    v: *const ReadPreferenceT,
) {
    safe_as_mut!(opts).0.selection_criteria = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_collection_options_set_server_selector(
    opts: *mut CollectionOptionsT,
    v: *const ServerSelectorT,
) {
    safe_as_mut!(opts).0.selection_criteria = safe_optional_as_ref!(v).map(Into::into);
}

impl From<&CollectionOptionsT> for CollectionOptions {
    fn from(value: &CollectionOptionsT) -> Self {
        value.0.clone()
    }
}
