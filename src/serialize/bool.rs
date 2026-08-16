// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct Bool {
    ptr: *mut pyo3::ffi::PyObject,
}

impl Bool {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        Bool { ptr: ptr }
    }
}

impl Serialize for Bool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = unsafe { self.ptr == pyo3::ffi::Py_True() };
        serializer.serialize_bool(value)
    }
}
