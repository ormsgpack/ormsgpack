// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use serde::ser::{Serialize, Serializer};

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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let contents = self.obj.as_bytes();
        serializer.serialize_bytes(contents)
    }
}
