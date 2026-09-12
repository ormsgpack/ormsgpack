// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::BorrowedWithType;
use pyo3::prelude::*;
use pyo3::types::PyFloat;
use serde::ser::{Serialize, Serializer};

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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = self.obj.value();
        serializer.serialize_f64(value)
    }
}
