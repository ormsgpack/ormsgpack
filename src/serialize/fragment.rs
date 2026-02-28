// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::pybytes_as_bytes;
use crate::fragment::PyFragment;
use serde::ser::{Serialize, Serializer};
use serde_bytes::Bytes;

#[repr(transparent)]
pub struct Fragment {
    ptr: *mut pyo3::ffi::PyObject,
}

impl Fragment {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        Fragment { ptr: ptr }
    }
}

impl Serialize for Fragment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let fragment = self.ptr.cast::<PyFragment>();
        let data = unsafe { pybytes_as_bytes((*fragment).data) };

        serializer.serialize_newtype_struct("", Bytes::new(data))
    }
}
