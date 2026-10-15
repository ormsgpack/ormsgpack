// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::ffi::*;
use std::ptr::NonNull;

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
        let len = unsafe { PyDict_Size(self.op) } as usize;
        (len, Some(len))
    }
}
