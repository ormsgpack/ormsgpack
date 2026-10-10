// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::int::*;
use pyo3::ffi::*;
use pyo3::prelude::*;
use pyo3::types::PyInt;

#[repr(C)]
#[cfg(Py_3_12)]
struct _PyLongValue {
    pub lv_tag: usize,
}

#[repr(C)]
#[cfg(Py_3_12)]
struct PyLongObject {
    pub ob_base: PyObject,
    pub long_value: _PyLongValue,
}

#[cfg(Py_3_12)]
const SIGN_MASK: usize = 3;

#[cfg(Py_3_12)]
pub fn pylong_is_positive(obj: Borrowed<'_, '_, PyInt>) -> bool {
    let op = obj.as_ptr();
    let tag = unsafe { (*op.cast::<PyLongObject>()).long_value.lv_tag };
    tag & SIGN_MASK == 0
}

#[cfg(not(Py_3_12))]
pub fn pylong_is_positive(obj: Borrowed<'_, '_, PyInt>) -> bool {
    let op = obj.as_ptr();
    let size = unsafe { (*op.cast::<PyVarObject>()).ob_size };
    size > 0
}

impl Int {
    pub fn new(obj: Borrowed<'_, '_, PyInt>) -> Result<Self, IntError> {
        if pylong_is_positive(obj) {
            match pylong_to_u64(obj) {
                Some(val) => Ok(Int::Unsigned(val)),
                None => Err(IntError::Overflow),
            }
        } else {
            match pylong_to_i64(obj) {
                Some(val) => Ok(Int::Signed(val)),
                None => Err(IntError::Overflow),
            }
        }
    }
}
