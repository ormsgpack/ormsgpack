// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::unicode::*;
use pyo3::ffi::*;
use pyo3::prelude::*;
use pyo3::types::PyString;

#[inline(always)]
pub fn hash_str(obj: Borrowed<'_, '_, PyString>) -> Py_hash_t {
    unsafe { PyObject_Hash(obj.as_ptr()) }
}

#[inline(always)]
pub fn unicode_from_str<'py>(py: Python<'py>, buf: &str) -> Bound<'py, PyString> {
    PyString::new(py, buf)
}

#[inline(always)]
pub fn unicode_to_str<'a>(obj: Borrowed<'a, '_, PyString>) -> Result<&'a str, UnicodeError> {
    unicode_to_str_via_ffi(obj)
}
