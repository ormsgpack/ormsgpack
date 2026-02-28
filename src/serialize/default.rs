// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
use crate::msgpack::RECURSION_LIMIT;

use std::cell::Cell;
use std::ffi::CStr;
use std::ptr::NonNull;

pub enum Error {
    InvalidType(*mut pyo3::ffi::PyObject),
    RecursionLimitReached,
}

impl std::fmt::Display for Error {
    #[cold]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::InvalidType(ptr) => {
                let name = unsafe { CStr::from_ptr((*ob_type!(ptr)).tp_name).to_string_lossy() };
                write!(f, "Type is not msgpack serializable: {name}")
            }
            Error::RecursionLimitReached => f.write_str("Recursion limit for default hook reached"),
        }
    }
}

pub struct DefaultHook {
    pub inner: Option<NonNull<pyo3::ffi::PyObject>>,
    recursion: Cell<u8>,
}

impl DefaultHook {
    pub fn new(default: Option<NonNull<pyo3::ffi::PyObject>>) -> Self {
        DefaultHook {
            inner: default,
            recursion: Cell::new(0),
        }
    }

    pub fn enter_call(
        &self,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> Result<*mut pyo3::ffi::PyObject, Error> {
        match self.inner {
            Some(callable) => {
                let recursion = self.recursion.get();
                if unlikely!(recursion == RECURSION_LIMIT) {
                    return Err(Error::RecursionLimitReached);
                }
                self.recursion.set(recursion + 1);
                let default_obj = unsafe { pyobject_call_one_arg(callable.as_ptr(), ptr) };
                if unlikely!(default_obj.is_null()) {
                    Err(Error::InvalidType(ptr))
                } else {
                    Ok(default_obj)
                }
            }
            None => Err(Error::InvalidType(ptr)),
        }
    }

    pub fn leave_call(&self) {
        let recursion = self.recursion.get();
        self.recursion.set(recursion - 1);
    }
}
