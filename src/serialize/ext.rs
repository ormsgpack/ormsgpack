// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ext::PyExt;
use crate::ffi::BorrowedWithType;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyInt};
use serde::ser::{Serialize, Serializer};
use serde_bytes::Bytes;

pub struct State {
    pub type_object: Py<PyAny>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            type_object: crate::ext::create_ext_type(py)?.unbind(),
        })
    }
}

#[repr(transparent)]
pub struct Ext<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
}

impl<'a, 'py> Ext<'a, 'py> {
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

impl Serialize for Ext<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ext = self.obj.as_ptr().cast::<PyExt>();
        let tag = match unsafe {
            Borrowed::from_ptr(self.obj.py(), (*ext).tag).cast_unchecked::<PyInt>()
        }
        .extract::<u32>()
        {
            Ok(tag @ 0..=127) => tag,
            _ => return Err(serde::ser::Error::custom("Extension type out of range")),
        };
        let data =
            unsafe { Borrowed::from_ptr(self.obj.py(), (*ext).data).cast_unchecked::<PyBytes>() };

        serializer.serialize_newtype_variant("", tag, "", Bytes::new(data.as_bytes()))
    }
}
