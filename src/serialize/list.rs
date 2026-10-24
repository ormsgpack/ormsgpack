// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::sync::critical_section::with_critical_section;
use pyo3::types::PyList;

#[repr(transparent)]
pub struct List<'a, 'py> {
    obj: Borrowed<'a, 'py, PyList>,
}

impl<'a, 'py> List<'a, 'py> {
    #[inline]
    pub fn try_new_exact(obj: BorrowedWithType<'a, 'py>) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyList>()?,
        })
    }

    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>) -> Option<Self> {
        Some(Self {
            obj: obj.cast::<PyList>()?,
        })
    }
}

impl Serialize for List<'_, '_> {
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        with_critical_section(&self.obj, || {
            let len = self.obj.len();
            serializer.serialize_seq(len, |seq| {
                for i in 0..len {
                    let item = unsafe {
                        let item = pyo3::ffi::PyList_GET_ITEM(self.obj.as_ptr(), i as isize);
                        Borrowed::from_ptr(self.obj.py(), item)
                    };
                    seq.serialize_element(&item)?;
                }
                Ok(())
            })
        })
    }
}
