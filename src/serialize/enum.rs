// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::{get_type, BorrowedWithType};
use crate::io::WriteSlices;
use crate::serialize::{Error, Serialize, Serializer};

use pyo3::prelude::*;
use pyo3::types::PyString;

pub struct State {
    type_object: Py<PyAny>,
    value_str: Py<PyString>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            type_object: py.import("enum")?.getattr("EnumMeta")?.unbind(),
            value_str: PyString::intern(py, "value").unbind(),
        })
    }
}

pub struct Enum<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
    state: &'a State,
}

impl<'a, 'py> Enum<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, state: &'a State) -> Option<Self> {
        if get_type(obj.get_type()).as_type_ptr() == state.type_object.as_ptr().cast() {
            Some(Self {
                obj: obj.as_borrowed(),
                state: state,
            })
        } else {
            None
        }
    }
}

impl Serialize for Enum<'_, '_> {
    fn serialize_as<const KEY: bool, W: WriteSlices>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error> {
        let value = self
            .obj
            .getattr(self.state.value_str.bind_borrowed(self.obj.py()))?;
        value.as_borrowed().serialize_as::<KEY, W>(serializer)
    }
}
