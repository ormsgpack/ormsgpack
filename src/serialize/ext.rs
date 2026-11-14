// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ext::PyExt;
use crate::ffi::BorrowedWithType;
use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::PyType;

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
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let py = self.obj.py();
        let ext = self.obj.get();
        let tag = match ext.tag.bind(py).extract::<u32>() {
            Ok(tag @ 0..=127) => tag,
            _ => return Err(Error::custom("Extension type out of range")),
        };
        let data = ext.data.bind(py);

        serializer.serialize_ext(tag as i8, data.as_bytes())
    }
}
