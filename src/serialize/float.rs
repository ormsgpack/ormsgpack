// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::PyFloat;

#[repr(transparent)]
pub struct Float<'a, 'py> {
    obj: Borrowed<'a, 'py, PyFloat>,
}

impl<'a, 'py> Float<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyFloat>()?,
        })
    }
}

impl Serialize for Float<'_, '_> {
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let value = self.obj.value();
        serializer.serialize_f64(value)
    }
}
