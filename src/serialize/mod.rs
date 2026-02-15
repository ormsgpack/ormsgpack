// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod bytearray;
mod bytes;
mod dataclass;
mod datetime;
mod datetimelike;
mod default;
mod dict;
mod ext;
mod list;
mod memoryview;
mod numpy;
mod pydantic;
mod serializer;
mod str;
mod tuple;
mod uuid;
mod writer;

pub use serializer::serialize;
