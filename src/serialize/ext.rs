// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ext::PyExt;
use crate::ffi::BorrowedWithType;
use pyo3::prelude::*;
use pyo3::types::PyType;
use serde::ser::{Serialize, Serializer};
use serde_bytes::Bytes;

pub struct State {
    pub type_object: Py<PyType>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            type_object: py.get_type::<PyExt>().unbind(),
        })
    }
}

#[repr(transparent)]
pub struct Ext<'a, 'py> {
    obj: Borrowed<'a, 'py, PyExt>,
}

impl<'a, 'py> Ext<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, state: &State) -> Option<Self> {
        if obj.get_type_ptr() == state.type_object.as_ptr().cast() {
            Some(Self {
                obj: unsafe { obj.as_borrowed().cast_unchecked::<PyExt>() },
            })
        } else {
            None
        }
    }
}

impl Serialize for Ext<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let py = self.obj.py();
        let ext = self.obj.get();
        let tag = match ext.tag.bind(py).extract::<u32>() {
            Ok(tag @ 0..=127) => tag,
            _ => return Err(serde::ser::Error::custom("Extension type out of range")),
        };
        let data = ext.data.bind(py);

        serializer.serialize_newtype_variant("", tag, "", Bytes::new(data.as_bytes()))
    }
}
