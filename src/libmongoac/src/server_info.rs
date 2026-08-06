use mongodb::ServerInfo;

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
pub extern "C" fn mongoac_server_info_get_type(_info: *const ServerInfoT) -> mongoac_server_type_t {
    unimplemented!();
}
