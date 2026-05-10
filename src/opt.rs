// SPDX-License-Identifier: (Apache-2.0 OR MIT)

pub type Opt = u16;

pub const NAIVE_UTC: Opt = 1;
pub const NON_STR_KEYS: Opt = 1 << 1;
pub const OMIT_MICROSECONDS: Opt = 1 << 2;
pub const PASSTHROUGH_BIG_INT: Opt = 1 << 3;
pub const PASSTHROUGH_DATACLASS: Opt = 1 << 4;
pub const PASSTHROUGH_DATETIME: Opt = 1 << 5;
pub const PASSTHROUGH_SUBCLASS: Opt = 1 << 6;
pub const PASSTHROUGH_TUPLE: Opt = 1 << 7;
pub const SERIALIZE_NUMPY: Opt = 1 << 8;
pub const SERIALIZE_PYDANTIC: Opt = 1 << 9;
pub const SORT_KEYS: Opt = 1 << 10;
pub const UTC_Z: Opt = 1 << 11;
pub const PASSTHROUGH_UUID: Opt = 1 << 12;
pub const PASSTHROUGH_ENUM: Opt = 1 << 13;
pub const DATETIME_AS_TIMESTAMP_EXT: Opt = 1 << 14;
pub const REPLACE_SURROGATES: Opt = 1 << 15;

pub const PACKB_OPT_MASK: i32 = (DATETIME_AS_TIMESTAMP_EXT
    | NAIVE_UTC
    | NON_STR_KEYS
    | OMIT_MICROSECONDS
    | PASSTHROUGH_BIG_INT
    | PASSTHROUGH_DATACLASS
    | PASSTHROUGH_DATETIME
    | PASSTHROUGH_ENUM
    | PASSTHROUGH_SUBCLASS
    | PASSTHROUGH_TUPLE
    | PASSTHROUGH_UUID
    | REPLACE_SURROGATES
    | SERIALIZE_NUMPY
    | SERIALIZE_PYDANTIC
    | SORT_KEYS
    | UTC_Z) as i32;

pub const UNPACKB_OPT_MASK: i32 = (DATETIME_AS_TIMESTAMP_EXT | NON_STR_KEYS) as i32;
