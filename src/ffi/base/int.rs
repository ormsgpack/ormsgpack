// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::int::*;
use pyo3::prelude::*;
use pyo3::types::PyInt;

impl Int {
    pub fn new(obj: Borrowed<'_, '_, PyInt>) -> Result<Self, IntError> {
        match pylong_to_i64(obj) {
            Some(val) => Ok(Int::Signed(val)),
            None => match pylong_to_u64(obj) {
                Some(val) => Ok(Int::Unsigned(val)),
                None => Err(IntError::Overflow),
            },
        }
    }
}
