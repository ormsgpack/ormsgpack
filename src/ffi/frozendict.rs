// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::ffi::*;
use pyo3::prelude::*;
use pyo3::types::PyFrozenDict;

pub struct PyFrozenDictIter<'a, 'py> {
    obj: Borrowed<'a, 'py, PyFrozenDict>,
    pos: isize,
}

impl<'a, 'py> PyFrozenDictIter<'a, 'py> {
    #[inline]
    pub fn new(obj: Borrowed<'a, 'py, PyFrozenDict>) -> Self {
        Self { obj: obj, pos: 0 }
    }
}

impl<'a, 'py> Iterator for PyFrozenDictIter<'a, 'py> {
    type Item = (Borrowed<'a, 'py, PyAny>, Borrowed<'a, 'py, PyAny>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut key: *mut PyObject = std::ptr::null_mut();
        let mut value: *mut PyObject = std::ptr::null_mut();
        unsafe {
            if PyDict_Next(self.obj.as_ptr(), &mut self.pos, &mut key, &mut value) == 1 {
                Some((
                    Borrowed::from_ptr(self.obj.py(), key),
                    Borrowed::from_ptr(self.obj.py(), value),
                ))
            } else {
                None
            }
        }
    }
}
