use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;

use mongodb::bson::deserialize_from_slice;
use mongodb::options::{
    HedgedReadOptions, ReadPreference, ReadPreferenceOptions, SelectionCriteria, TagSet,
};
use std::time::Duration;

pub struct ReadPreferenceT(ReadPreference);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_new() -> *mut ReadPreferenceT {
    Box::into_raw(Box::new(ReadPreferenceT(ReadPreference::Primary)))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_destroy(rp: *mut ReadPreferenceT) {
    safe_drop!(rp);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_set_primary(rp: *mut ReadPreferenceT) {
    safe_as_mut!(rp).0 = ReadPreference::Primary;
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_set_secondary(rp: *mut ReadPreferenceT) {
    let rp = safe_as_mut!(rp);

    rp.0 = ReadPreference::Secondary {
        options: take_options(rp),
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_set_primary_preferred(rp: *mut ReadPreferenceT) {
    let rp = safe_as_mut!(rp);

    rp.0 = ReadPreference::PrimaryPreferred {
        options: take_options(rp),
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_set_secondary_preferred(rp: *mut ReadPreferenceT) {
    let rp = safe_as_mut!(rp);

    rp.0 = ReadPreference::SecondaryPreferred {
        options: take_options(rp),
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_set_nearest(rp: *mut ReadPreferenceT) {
    let rp = safe_as_mut!(rp);

    rp.0 = ReadPreference::Nearest {
        options: take_options(rp),
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_set_max_staleness_seconds(
    rp: *mut ReadPreferenceT,
    v: u64,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let rp = safe_as_mut_with_error!(rp, error);

    if let Some(options) = get_options_as_mut(&mut rp.0) {
        options.get_or_insert_with(Default::default).max_staleness = Some(Duration::from_secs(v));
    } else {
        crate::private::safety::invalid_argument(
            error,
            "read preference mode 'primary' does not support maxStalenessSeconds",
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_add_tag_set(
    rp: *mut ReadPreferenceT,
    tag_set: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let rp = safe_as_mut_with_error!(rp, error);

    if matches!(rp.0, ReadPreference::Primary) {
        crate::private::safety::invalid_argument(
            error,
            "read preference mode 'primary' does not support tag sets",
        );
        return;
    }

    let tag_set = safe_bson_view_with_error!(tag_set, error);
    let ts: TagSet = safe_error!(deserialize_from_slice(tag_set.as_bytes()), error);

    if let Some(options) = get_options_as_mut(&mut rp.0) {
        options
            .get_or_insert_with(Default::default)
            .tag_sets
            .get_or_insert_default()
            .push(ts);
    } else {
        crate::private::safety::invalid_argument(
            error,
            "read preference mode 'primary' does not support tag sets",
        );
    }
}

#[allow(deprecated)]
#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_preference_set_hedge_enabled(
    rp: *mut ReadPreferenceT,
    v: bool,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let rp = safe_as_mut_with_error!(rp, error);

    if let Some(options) = get_options_as_mut(&mut rp.0) {
        options.get_or_insert_with(Default::default).hedge =
            Some(HedgedReadOptions::builder().enabled(v).build());
    } else {
        crate::private::safety::invalid_argument(
            error,
            "read preference mode 'primary' does not support hedged reads",
        );
    }
}

impl From<&ReadPreferenceT> for SelectionCriteria {
    fn from(value: &ReadPreferenceT) -> Self {
        value.0.clone().into()
    }
}

fn take_options(rp: &mut ReadPreferenceT) -> Option<ReadPreferenceOptions> {
    match std::mem::replace(&mut rp.0, ReadPreference::Primary) {
        ReadPreference::Secondary { options }
        | ReadPreference::PrimaryPreferred { options }
        | ReadPreference::SecondaryPreferred { options }
        | ReadPreference::Nearest { options } => options,
        ReadPreference::Primary | _ => None, // #[non_exhaustive]
    }
}

fn get_options_as_mut(rp: &mut ReadPreference) -> Option<&mut Option<ReadPreferenceOptions>> {
    match rp {
        ReadPreference::Secondary { options }
        | ReadPreference::PrimaryPreferred { options }
        | ReadPreference::SecondaryPreferred { options }
        | ReadPreference::Nearest { options } => Some(options),
        ReadPreference::Primary | _ => None, // #[non_exhaustive]
    }
}
