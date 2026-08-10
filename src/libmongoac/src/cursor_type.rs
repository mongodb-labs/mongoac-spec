use mongodb::options::CursorType;

// C enum for cursor types, exposed as #define constants.
// See README.md "Why BSON string for CursorType?" — this is the first options struct
// to realize the spec's CursorType end-state (typed #define enum + typed setter).
// find_options will later reuse these constants when it migrates from new_from_bson().

#[allow(non_camel_case_types)]
pub type mongoac_cursor_type_t = i32;
pub const MONGOAC_CURSOR_TYPE_NON_TAILABLE: mongoac_cursor_type_t = 0;
pub const MONGOAC_CURSOR_TYPE_TAILABLE: mongoac_cursor_type_t = 1;
pub const MONGOAC_CURSOR_TYPE_TAILABLE_AWAIT: mongoac_cursor_type_t = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(i32)]
pub enum CursorTypeT {
    NonTailable = MONGOAC_CURSOR_TYPE_NON_TAILABLE,
    Tailable = MONGOAC_CURSOR_TYPE_TAILABLE,
    TailableAwait = MONGOAC_CURSOR_TYPE_TAILABLE_AWAIT,

    #[num_enum(catch_all)]
    Unused(i32) = i32::MIN, // Never used: unknown values are mapped to `NonTailable`.
}

impl From<CursorTypeT> for CursorType {
    fn from(v: CursorTypeT) -> Self {
        match v {
            CursorTypeT::NonTailable | CursorTypeT::Unused(_) => Self::NonTailable, // #[non_exhaustive]
            CursorTypeT::Tailable => Self::Tailable,
            CursorTypeT::TailableAwait => Self::TailableAwait,
        }
    }
}
