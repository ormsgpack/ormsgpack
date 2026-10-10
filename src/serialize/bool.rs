// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use pyo3::prelude::*;
use pyo3::types::PyBool;
use serde::ser::{Serialize, Serializer};

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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = self.obj.is_true();
        serializer.serialize_bool(value)
    }
}
