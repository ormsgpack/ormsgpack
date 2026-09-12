// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use crate::serialize::serializer::*;
use crate::serialize::{Context, DictKeyContext};

use pyo3::prelude::*;
use pyo3::types::PyTuple;
use serde::ser::{Serialize, SerializeSeq, Serializer};

pub struct Tuple<'a, 'py, C> {
    obj: Borrowed<'a, 'py, PyTuple>,
    context: C,
}

impl<'a, 'py, C> Tuple<'a, 'py, C> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, context: C) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyTuple>()?,
            context: context,
        })
    }
}

impl Serialize for Tuple<'_, '_, Context<'_, '_>> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = self.obj.len();
        let mut seq = serializer.serialize_seq(Some(len))?;
        for item in self.obj.iter_borrowed() {
            let value = PyObject::new(item, self.context);
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

impl Serialize for Tuple<'_, '_, DictKeyContext<'_>> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = self.obj.len();
        let mut seq = serializer.serialize_seq(Some(len))?;
        for item in self.obj.iter_borrowed() {
            let value = DictKey::new(item, self.context);
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}
