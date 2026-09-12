// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::{BorrowedWithType, Buffer};
use pyo3::prelude::*;
use pyo3::types::PyMemoryView;
use serde::ser::{Serialize, Serializer};

#[repr(transparent)]
pub struct MemoryView<'a, 'py> {
    obj: Borrowed<'a, 'py, PyMemoryView>,
}

impl<'a, 'py> MemoryView<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyMemoryView>()?,
        })
    }
}

impl Serialize for MemoryView<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(buffer) = Buffer::get(self.obj) {
            serializer.serialize_bytes(buffer.as_bytes())
        } else {
            Err(serde::ser::Error::custom(
                "Failed to get buffer from memoryview",
            ))
        }
    }
}
