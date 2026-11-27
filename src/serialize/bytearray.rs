// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::sync::critical_section::with_critical_section;
use pyo3::types::PyByteArray;

#[repr(transparent)]
pub struct ByteArray<'a, 'py> {
    obj: Borrowed<'a, 'py, PyByteArray>,
}

impl<'a, 'py> ByteArray<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyByteArray>()?,
        })
    }
}

impl Serialize for ByteArray<'_, '_> {
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        with_critical_section(&self.obj, || {
            let contents = unsafe { self.obj.as_bytes() };
            serializer.serialize_bytes(contents)
        })
    }
}
