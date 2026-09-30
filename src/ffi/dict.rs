// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::ffi::*;
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub struct PyDictIter<'a, 'py> {
    obj: Borrowed<'a, 'py, PyDict>,
    pos: isize,
}

impl<'a, 'py> PyDictIter<'a, 'py> {
    #[inline]
    pub fn new(obj: Borrowed<'a, 'py, PyDict>) -> Self {
        Self { obj: obj, pos: 0 }
    }
}

impl<'a, 'py> Iterator for PyDictIter<'a, 'py> {
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.obj.len();
        (len, Some(len))
    }
}
