use std::ffi::c_char;

use mongodb::bson::serialize_to_raw_document_buf;
use mongodb::options::ServerAddress;
use mongodb::{ServerInfo, ServerType};

use crate::bson::BsonT;
use crate::private::macros::*;
use crate::string::StringViewT;

#[allow(non_camel_case_types)]
pub type mongoac_server_type_t = i32;
pub const MONGOAC_SERVER_TYPE_UNKNOWN: mongoac_server_type_t = 0;
pub const MONGOAC_SERVER_TYPE_STANDALONE: mongoac_server_type_t = 1;
pub const MONGOAC_SERVER_TYPE_MONGOS: mongoac_server_type_t = 2;
pub const MONGOAC_SERVER_TYPE_RS_PRIMARY: mongoac_server_type_t = 3;
pub const MONGOAC_SERVER_TYPE_RS_SECONDARY: mongoac_server_type_t = 4;
pub const MONGOAC_SERVER_TYPE_RS_ARBITER: mongoac_server_type_t = 5;
pub const MONGOAC_SERVER_TYPE_RS_OTHER: mongoac_server_type_t = 6;
pub const MONGOAC_SERVER_TYPE_RS_GHOST: mongoac_server_type_t = 7;
pub const MONGOAC_SERVER_TYPE_LOAD_BALANCER: mongoac_server_type_t = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(i32)]
pub enum ServerTypeT {
    Unknown = MONGOAC_SERVER_TYPE_UNKNOWN,
    Standalone = MONGOAC_SERVER_TYPE_STANDALONE,
    Mongos = MONGOAC_SERVER_TYPE_MONGOS,
    RsPrimary = MONGOAC_SERVER_TYPE_RS_PRIMARY,
    RsSecondary = MONGOAC_SERVER_TYPE_RS_SECONDARY,
    RsArbiter = MONGOAC_SERVER_TYPE_RS_ARBITER,
    RsOther = MONGOAC_SERVER_TYPE_RS_OTHER,
    RsGhost = MONGOAC_SERVER_TYPE_RS_GHOST,
    LoadBalancer = MONGOAC_SERVER_TYPE_LOAD_BALANCER,

    #[num_enum(catch_all)]
    Unused(i32) = i32::MIN, // Never used: unknown values are mapped to `Unknown`.
}

pub struct ServerInfoT<'a>(ServerInfo<'a>);

impl<'a> From<&ServerInfo<'a>> for ServerInfoT<'a> {
    fn from(info: &ServerInfo<'a>) -> Self {
        ServerInfoT(info.clone())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_type(info: *const ServerInfoT) -> mongoac_server_type_t {
    ServerTypeT::from(safe_as_ref!(info).0.server_type()).into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_host(info: *const ServerInfoT) -> StringViewT {
    match safe_as_ref!(info).0.address() {
        ServerAddress::Tcp { host, .. } => host.as_str().into(),
        #[cfg(unix)]
        ServerAddress::Unix { path } => {
            use std::os::unix::ffi::OsStrExt;
            let bytes = path.as_os_str().as_bytes();
            StringViewT {
                data: bytes.as_ptr().cast::<c_char>(),
                len: bytes.len(),
            }
        }
        _ => StringViewT::default(), // #[non_exhaustive]
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_port(info: *const ServerInfoT) -> u16 {
    match safe_as_ref!(info).0.address() {
        ServerAddress::Tcp { port, .. } => port.unwrap_or(DEFAULT_PORT),
        #[cfg(unix)]
        ServerAddress::Unix { .. } => 0,
        _ => Default::default(), // #[non_exhaustive]
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_average_round_trip_time_secs(
    info: *const ServerInfoT,
) -> u64 {
    safe_as_ref!(info)
        .0
        .average_round_trip_time()
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_average_round_trip_time_nanos(
    info: *const ServerInfoT,
) -> u32 {
    safe_as_ref!(info)
        .0
        .average_round_trip_time()
        .map(|d| d.subsec_nanos())
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_average_round_trip_time_has_value(
    info: *const ServerInfoT,
) -> bool {
    safe_as_ref!(info).0.average_round_trip_time().is_some()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_last_update_time(info: *const ServerInfoT) -> i64 {
    safe_as_ref!(info)
        .0
        .last_update_time()
        .map(mongodb::bson::DateTime::timestamp_millis)
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_last_update_time_has_value(info: *const ServerInfoT) -> bool {
    safe_as_ref!(info).0.last_update_time().is_some()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_max_wire_version(info: *const ServerInfoT) -> i32 {
    safe_as_ref!(info).0.max_wire_version().unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_max_wire_version_has_value(info: *const ServerInfoT) -> bool {
    safe_as_ref!(info).0.max_wire_version().is_some()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_min_wire_version(info: *const ServerInfoT) -> i32 {
    safe_as_ref!(info).0.min_wire_version().unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_min_wire_version_has_value(info: *const ServerInfoT) -> bool {
    safe_as_ref!(info).0.min_wire_version().is_some()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_replica_set_name(
    info: *const ServerInfoT,
) -> StringViewT {
    safe_as_ref!(info)
        .0
        .replica_set_name()
        .map(StringViewT::from)
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_replica_set_version(info: *const ServerInfoT) -> i32 {
    safe_as_ref!(info)
        .0
        .replica_set_version()
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_replica_set_version_has_value(
    info: *const ServerInfoT,
) -> bool {
    safe_as_ref!(info).0.replica_set_version().is_some()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_get_tags(info: *const ServerInfoT) -> BsonT {
    safe_as_ref!(info)
        .0
        .tags()
        .and_then(|tags| serialize_to_raw_document_buf(tags).ok())
        .map(Into::into)
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_info_has_error(info: *const ServerInfoT) -> bool {
    safe_as_ref!(info).0.error().is_some()
}

// Same as `mongodb::client::options::DEFAULT_PORT`.
const DEFAULT_PORT: u16 = 27017;

impl From<ServerType> for ServerTypeT {
    fn from(t: ServerType) -> Self {
        match t {
            ServerType::Standalone => Self::Standalone,
            ServerType::Mongos => Self::Mongos,
            ServerType::RsPrimary => Self::RsPrimary,
            ServerType::RsSecondary => Self::RsSecondary,
            ServerType::RsArbiter => Self::RsArbiter,
            ServerType::RsOther => Self::RsOther,
            ServerType::RsGhost => Self::RsGhost,
            ServerType::LoadBalancer => Self::LoadBalancer,
            ServerType::Unknown | _ => Self::Unknown, // #[non_exhaustive]
        }
    }
}
