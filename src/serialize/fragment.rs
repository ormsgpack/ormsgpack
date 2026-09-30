// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use crate::fragment::PyFragment;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use serde::ser::{Serialize, Serializer};
use serde_bytes::Bytes;

pub struct State {
    pub type_object: Py<PyAny>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            type_object: crate::fragment::create_fragment_type(py)?.unbind(),
        })
    }
}

#[repr(transparent)]
pub struct Fragment<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
}

impl<'a, 'py> Fragment<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, state: &State) -> Option<Self> {
        if obj.get_type_ptr() == state.type_object.as_ptr().cast() {
            Some(Self {
                obj: obj.as_borrowed(),
            })
        } else {
            None
        }
    }
}

impl Serialize for Fragment<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let fragment = self.obj.as_ptr().cast::<PyFragment>();
        let data = unsafe {
            Borrowed::from_ptr(self.obj.py(), (*fragment).data).cast_unchecked::<PyBytes>()
        };

        serializer.serialize_newtype_struct("", Bytes::new(data.as_bytes()))
    }
}
