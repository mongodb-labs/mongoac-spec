use crate::private::macros::*;
use crate::write_concern::WriteConcernT;

use mongodb::options::DropCollectionOptions;

pub struct DropCollectionOptionsT(DropCollectionOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_drop_collection_options_new() -> *mut DropCollectionOptionsT {
    Box::into_raw(Box::new(DropCollectionOptionsT(
        DropCollectionOptions::default(),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_drop_collection_options_destroy(opts: *mut DropCollectionOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_drop_collection_options_set_write_concern(
    opts: *mut DropCollectionOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

impl From<&DropCollectionOptionsT> for DropCollectionOptions {
    fn from(value: &DropCollectionOptionsT) -> Self {
        value.0.clone()
    }
}
