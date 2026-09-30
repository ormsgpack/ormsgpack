// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::get_type_name;
use crate::msgpack::RECURSION_LIMIT;
use crate::util::unlikely;

use pyo3::prelude::*;
use std::cell::Cell;

pub enum Error {
    InvalidType(String),
    RecursionLimitReached,
}

impl std::fmt::Display for Error {
    #[cold]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidType(name) => write!(f, "Type is not msgpack serializable: {name}"),
            Error::RecursionLimitReached => f.write_str("Recursion limit for default hook reached"),
        }
    }
}

pub struct DefaultHook<'a, 'py> {
    pub inner: Option<Borrowed<'a, 'py, PyAny>>,
    recursion: Cell<u8>,
}

impl<'a, 'py> DefaultHook<'a, 'py> {
    pub fn new(default: Option<Borrowed<'a, 'py, PyAny>>) -> Self {
        DefaultHook {
            inner: default,
            recursion: Cell::new(0),
        }
    }

    pub fn enter_call(&self, obj: Borrowed<'_, 'py, PyAny>) -> Result<Bound<'py, PyAny>, Error> {
        match &self.inner {
            Some(callable) => {
                let recursion = self.recursion.get();
                if unlikely(recursion == RECURSION_LIMIT) {
                    return Err(Error::RecursionLimitReached);
                }
                self.recursion.set(recursion + 1);
                callable
                    .call1((obj,))
                    .map_err(|_| Error::InvalidType(get_type_name(obj)))
            }
            None => Err(Error::InvalidType(get_type_name(obj))),
        }
    }

    pub fn leave_call(&self) {
        let recursion = self.recursion.get();
        self.recursion.set(recursion - 1);
    }
}
