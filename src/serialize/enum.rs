// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::{get_type, BorrowedWithType};
use crate::serialize::serializer::{DictKey, PyObject};
use crate::serialize::{pyerr_to_serde, Context, DictKeyContext};

use pyo3::prelude::*;
use pyo3::types::PyString;
use serde::ser::{Serialize, Serializer};

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

pub struct Enum<'a, 'py, C> {
    obj: Borrowed<'a, 'py, PyAny>,
    context: C,
}

impl<'a, 'py, C> Enum<'a, 'py, C> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, state: &State, context: C) -> Option<Self> {
        if get_type(obj.get_type()).as_type_ptr() == state.type_object.as_ptr().cast() {
            Some(Self {
                obj: obj.as_borrowed(),
                context: context,
            })
        } else {
            None
        }
    }
}

impl Serialize for Enum<'_, '_, Context<'_, '_>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = self
            .obj
            .getattr(
                self.context
                    .state
                    .enum_
                    .value_str
                    .bind_borrowed(self.obj.py()),
            )
            .map_err(|e| pyerr_to_serde(self.obj.py(), e))?;
        PyObject::new(value.as_borrowed(), self.context).serialize(serializer)
    }
}

impl Serialize for Enum<'_, '_, DictKeyContext<'_>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = self
            .obj
            .getattr(
                self.context
                    .state
                    .enum_
                    .value_str
                    .bind_borrowed(self.obj.py()),
            )
            .map_err(|e| pyerr_to_serde(self.obj.py(), e))?;
        DictKey::new(value.as_borrowed(), self.context).serialize(serializer)
    }
}
