// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::PyBool;

#[repr(transparent)]
pub struct Bool<'a, 'py> {
    obj: Borrowed<'a, 'py, PyBool>,
}

impl<'a, 'py> Bool<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyBool>()?,
        })
    }
}

impl Serialize for Bool<'_, '_> {
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let value = self.obj.is_true();
        serializer.serialize_bool(value)
    }
}
