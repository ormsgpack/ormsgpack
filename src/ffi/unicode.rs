// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::util::unlikely;
use pyo3::ffi::*;
use pyo3::prelude::*;
use pyo3::types::PyString;

#[derive(Debug)]
pub enum UnicodeError {
    Surrogates,
}

impl std::fmt::Display for UnicodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surrogates => write!(f, "string contains surrogates"),
        }
    }
}

#[inline(never)]
pub fn unicode_to_str_via_ffi<'a>(
    obj: Borrowed<'a, '_, PyString>,
) -> Result<&'a str, UnicodeError> {
    unsafe {
        let mut size: Py_ssize_t = 0;
        let ptr = PyUnicode_AsUTF8AndSize(obj.as_ptr(), &mut size).cast::<u8>();
        if unlikely(ptr.is_null()) {
            PyErr_Clear();
            Err(UnicodeError::Surrogates)
        } else {
            let slice = std::slice::from_raw_parts(ptr, size as usize);
            Ok(std::str::from_utf8_unchecked(slice))
        }
    }
}
