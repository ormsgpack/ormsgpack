// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use crate::io::WriteSlices;
use crate::serialize::{Error, Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

#[repr(transparent)]
pub struct Tuple<'a, 'py> {
    obj: Borrowed<'a, 'py, PyTuple>,
}

impl<'a, 'py> Tuple<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyTuple>()?,
        })
    }
}

impl Serialize for Tuple<'_, '_> {
    #[inline(never)]
    fn serialize_as<const KEY: bool, W: WriteSlices>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error> {
        serializer.serialize_seq(self.obj.len(), |seq| {
            for item in self.obj.iter_borrowed() {
                seq.serialize_element_as::<KEY, _>(&item)?;
            }
            Ok(())
        })
    }
}
