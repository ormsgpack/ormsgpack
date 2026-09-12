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

use crate::opt::Opt;
use default::DefaultHook;
use pyo3::prelude::*;

#[cold]
fn pyerr_to_serde<E>(py: Python<'_>, err: PyErr) -> E
where
    E: serde::ser::Error,
{
    E::custom(err.value(py).to_string())
}

#[derive(Clone, Copy)]
pub struct Context<'a, 'py> {
    pub state: &'a State,
    pub opts: Opt,
    pub default: &'a DefaultHook<'a, 'py>,
}

#[derive(Clone, Copy)]
pub struct DictKeyContext<'a> {
    pub state: &'a State,
    pub opts: Opt,
}

pub use serializer::serialize;
pub use state::State;
