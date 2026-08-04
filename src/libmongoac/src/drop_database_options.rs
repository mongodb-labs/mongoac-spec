use crate::private::macros::*;
use crate::write_concern::WriteConcernT;

use mongodb::options::DropDatabaseOptions;

pub struct DropDatabaseOptionsT(DropDatabaseOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_drop_database_options_new() -> *mut DropDatabaseOptionsT {
    Box::into_raw(Box::new(DropDatabaseOptionsT(
        DropDatabaseOptions::default(),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_drop_database_options_destroy(opts: *mut DropDatabaseOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_drop_database_options_set_write_concern(
    opts: *mut DropDatabaseOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

impl From<&DropDatabaseOptionsT> for DropDatabaseOptions {
    fn from(value: &DropDatabaseOptionsT) -> Self {
        value.0.clone()
    }
}
