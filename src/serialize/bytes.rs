// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[repr(transparent)]
pub struct Bytes<'a, 'py> {
    obj: Borrowed<'a, 'py, PyBytes>,
}

impl<'a, 'py> Bytes<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyBytes>()?,
        })
    }
}

impl Serialize for Bytes<'_, '_> {
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let contents = self.obj.as_bytes();
        serializer.serialize_bytes(contents)
    }
}
