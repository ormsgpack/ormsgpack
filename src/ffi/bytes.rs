// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::ffi::*;
use pyo3::prelude::*;

#[repr(transparent)]
pub struct Buffer(Py_buffer);

impl Buffer {
    pub fn get<T>(obj: Borrowed<'_, '_, T>) -> Option<Self> {
        unsafe {
            let mut view: Py_buffer = std::mem::zeroed();
            if PyObject_GetBuffer(obj.as_ptr(), &mut view, PyBUF_CONTIG_RO) == -1 {
                None
            } else {
                Some(Self(view))
            }
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        let buffer = self.0.buf.cast::<u8>();
        let length = self.0.len as usize;
        unsafe { std::slice::from_raw_parts(buffer, length) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { PyBuffer_Release(&mut self.0) }
    }
}
