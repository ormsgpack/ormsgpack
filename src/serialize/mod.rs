// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[path = "bool.rs"]
mod bool_;
mod bytearray;
mod bytes;
mod dataclass;
mod datetime;
mod datetimelike;
mod default;
mod dict;
#[path = "enum.rs"]
mod enum_;
mod ext;
mod float;
mod fragment;
#[cfg(Py_3_15)]
mod frozendict;
mod list;
mod memoryview;
mod numpy;
mod pydantic;
mod serializer;
mod state;
mod str;
mod tuple;
mod uuid;
mod writer;

pub use serializer::{serialize, Error, Serialize, Serializer};
pub use state::State;
