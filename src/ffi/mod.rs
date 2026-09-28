// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod bytes;
mod critical_section;
mod dict;
#[cfg_attr(any(PyPy, GraalPy), path = "base/mod.rs")]
#[cfg_attr(not(any(PyPy, GraalPy)), path = "cpython/mod.rs")]
mod impl_;
mod int;
mod unicode;

pub use bytes::*;
pub use critical_section::*;
pub use dict::*;
pub use impl_::*;
pub use int::*;
pub use unicode::*;

use pyo3::ffi::*;

#[inline(always)]
pub unsafe fn pybytes_as_bytes(op: *mut PyObject) -> &'static [u8] {
    let buffer = pybytes_as_mut_u8(op);
    let length = Py_SIZE(op) as usize;
    std::slice::from_raw_parts(buffer, length)
}

#[inline(always)]
pub unsafe fn pybytearray_as_bytes(op: *mut PyObject) -> &'static [u8] {
    let buffer = PyByteArray_AsString(op).cast::<u8>();
    let length = PyByteArray_Size(op) as usize;
    std::slice::from_raw_parts(buffer, length)
}
