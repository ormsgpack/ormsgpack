// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct Float {
    ptr: *mut pyo3::ffi::PyObject,
}

impl Float {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        Float { ptr: ptr }
    }
}

impl Serialize for Float {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = unsafe { pyo3::ffi::PyFloat_AS_DOUBLE(self.ptr) };
        serializer.serialize_f64(value)
    }
}
