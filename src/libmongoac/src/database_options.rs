use crate::private::macros::*;
use crate::read_concern::ReadConcernT;
use crate::read_preference::ReadPreferenceT;
use crate::write_concern::WriteConcernT;

use mongodb::options::DatabaseOptions;

pub struct DatabaseOptionsT(DatabaseOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_options_new() -> *mut DatabaseOptionsT {
    Box::into_raw(Box::new(DatabaseOptionsT(DatabaseOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_options_destroy(opts: *mut DatabaseOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_options_set_read_concern(
    opts: *mut DatabaseOptionsT,
    v: *const ReadConcernT,
) {
    safe_as_mut!(opts).0.read_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_options_set_write_concern(
    opts: *mut DatabaseOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_database_options_set_read_preference(
    opts: *mut DatabaseOptionsT,
    v: *const ReadPreferenceT,
) {
    safe_as_mut!(opts).0.selection_criteria = safe_optional_as_ref!(v).map(Into::into);
}

impl From<&DatabaseOptionsT> for DatabaseOptions {
    fn from(value: &DatabaseOptionsT) -> Self {
        value.0.clone()
    }
}
