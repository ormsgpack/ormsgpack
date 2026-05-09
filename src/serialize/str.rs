// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
use crate::opt::*;
use crate::util::unlikely;

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
struct StrWithSurrogates {
    ptr: *mut pyo3::ffi::PyObject,
}

impl StrWithSurrogates {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        StrWithSurrogates { ptr: ptr }
    }
}

impl Serialize for StrWithSurrogates {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            let ptr = pyo3::ffi::PyUnicode_AsEncodedString(
                self.ptr,
                c"UTF-8".as_ptr(),
                c"replace".as_ptr(),
            );
            if unlikely(ptr.is_null()) {
                return Err(serde::ser::Error::custom("invalid string"));
            }
            let slice = pybytes_as_bytes(ptr);
            let uni = std::str::from_utf8_unchecked(slice);
            let res = serializer.serialize_str(uni);
            pyo3::ffi::Py_DECREF(ptr);
            res
        }
    }
}

pub struct Str {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
}

impl Str {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, opts: Opt) -> Self {
        Str {
            ptr: ptr,
            opts: opts,
        }
    }
}

impl Serialize for Str {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match unicode_to_str(self.ptr) {
            Ok(val) => serializer.serialize_str(val),
            Err(err) => {
                if self.opts & REPLACE_SURROGATES != 0 {
                    StrWithSurrogates::new(self.ptr).serialize(serializer)
                } else {
                    Err(serde::ser::Error::custom(err))
                }
            }
        }
    }
}

pub struct StrSubclass {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
}

impl StrSubclass {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, opts: Opt) -> Self {
        StrSubclass {
            ptr: ptr,
            opts: opts,
        }
    }
}

impl Serialize for StrSubclass {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match unicode_to_str_via_ffi(self.ptr) {
            Ok(val) => serializer.serialize_str(val),
            Err(err) => {
                if self.opts & REPLACE_SURROGATES != 0 {
                    StrWithSurrogates::new(self.ptr).serialize(serializer)
                } else {
                    Err(serde::ser::Error::custom(err))
                }
            }
        }
    }
}
