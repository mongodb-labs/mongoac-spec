use std::ffi::c_void;
use std::sync::Arc;

use mongodb::ServerInfo;
use mongodb::options::SelectionCriteria;

use crate::private::macros::*;
use crate::server_info::ServerInfoT;

// SAFETY:
//  - MUST NOT invoke a progress function for the runtime which invoked the callback (no re-entrancy).
//  - MUST NOT panic, unwind, terminate, or otherwise fail.
//  - SHOULD NOT block by doing significant work.
#[allow(non_camel_case_types)]
pub type mongoac_server_predicate_t =
    Option<extern "C" fn(info: *const ServerInfoT, user_data: *mut c_void) -> bool>;

pub struct ServerSelectorT {
    predicate: mongoac_server_predicate_t,
    user_data: *mut c_void,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_selector_new(
    callback: mongoac_server_predicate_t,
    user_data: *mut c_void,
) -> *mut ServerSelectorT {
    if callback.is_none() {
        return Default::default();
    }

    Box::into_raw(Box::new(ServerSelectorT {
        predicate: callback,
        user_data,
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_selector_destroy(sc: *mut ServerSelectorT) {
    safe_drop!(sc);
}

// Required by `mongodb::options::SelectionCriteria::Predicate`.
unsafe impl Send for ServerSelectorT {}

// Required by `mongodb::options::SelectionCriteria::Predicate`.
unsafe impl Sync for ServerSelectorT {}

impl From<&ServerSelectorT> for SelectionCriteria {
    fn from(sc: &ServerSelectorT) -> Self {
        let predicate = sc.predicate;
        let user_data = sc.user_data as usize;

        SelectionCriteria::Predicate(Arc::new(move |info: &ServerInfo| -> bool {
            let server_info: ServerInfoT = info.into();

            match predicate {
                Some(cb) => cb(std::ptr::from_ref(&server_info), user_data as *mut c_void),
                None => true,
            }
        }))
    }
}
