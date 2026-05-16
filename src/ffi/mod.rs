// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod critical_section;
#[cfg_attr(any(PyPy, GraalPy), path = "base/mod.rs")]
#[cfg_attr(not(any(PyPy, GraalPy)), path = "cpython/mod.rs")]
mod impl_;
mod int;
mod unicode;

pub use critical_section::*;
pub use impl_::*;
pub use int::*;
pub use unicode::*;

use pyo3::ffi::*;
use std::ptr::NonNull;

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

pub struct PyDictIter {
    op: *mut PyObject,
    pos: isize,
}

impl PyDictIter {
    #[inline]
    pub fn from_pyobject(op: *mut PyObject) -> Self {
        PyDictIter { op: op, pos: 0 }
    }
}

impl Iterator for PyDictIter {
    type Item = (NonNull<PyObject>, NonNull<PyObject>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut key: *mut PyObject = std::ptr::null_mut();
        let mut value: *mut PyObject = std::ptr::null_mut();
        unsafe {
            if PyDict_Next(self.op, &mut self.pos, &mut key, &mut value) == 1 {
                Some((NonNull::new_unchecked(key), NonNull::new_unchecked(value)))
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe { pydict_size(self.op) } as usize;
        (len, Some(len))
    }
}

pub struct Buffer {
    view: Py_buffer,
}

impl Buffer {
    pub unsafe fn get(obj: *mut PyObject) -> Option<Self> {
        let mut view: Py_buffer = std::mem::zeroed();
        if PyObject_GetBuffer(obj, &mut view, PyBUF_CONTIG_RO) == -1 {
            return None;
        }
        Some(Self { view })
    }

    pub fn as_bytes(&self) -> &[u8] {
        let buffer = self.view.buf.cast::<u8>();
        let length = self.view.len as usize;
        unsafe { std::slice::from_raw_parts(buffer, length) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { PyBuffer_Release(&mut self.view) }
    }
}
