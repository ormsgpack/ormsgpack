// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use crate::fragment::PyFragment;
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
            type_object: py.get_type::<PyFragment>().unbind(),
        })
    }
}

#[repr(transparent)]
pub struct Fragment<'a, 'py> {
    obj: Borrowed<'a, 'py, PyFragment>,
}

impl<'a, 'py> Fragment<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, state: &State) -> Option<Self> {
        if obj.get_type_ptr() == state.type_object.as_ptr().cast() {
            Some(Self {
                obj: unsafe { obj.as_borrowed().cast_unchecked::<PyFragment>() },
            })
        } else {
            None
        }
    }
}

impl Serialize for Fragment<'_, '_> {
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let py = self.obj.py();
        let fragment = self.obj.get();
        let data = fragment.data.bind(py);

        serializer.serialize_fragment(data.as_bytes())
    }
}
