// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::{BorrowedWithType, Buffer};
use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::PyMemoryView;

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
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        if let Some(buffer) = Buffer::get(self.obj) {
            serializer.serialize_bytes(buffer.as_bytes())
        } else {
            Err(Error::custom("Failed to get buffer from memoryview"))
        }
    }
}
