// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::io::WriteSlices;
use pyo3::ffi::*;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

const BUFFER_LENGTH: usize = 1024;

pub struct BytesWriter {
    cap: usize,
    len: usize,
    writer: *mut compat::PyBytesWriter,
    data: *mut u8,
}

impl BytesWriter {
    pub fn default() -> Self {
        unsafe {
            let writer = compat::PyBytesWriter_Create(BUFFER_LENGTH as isize);
            BytesWriter {
                cap: BUFFER_LENGTH,
                len: 0,
                writer: writer,
                data: compat::PyBytesWriter_GetData(writer).cast::<u8>(),
            }
        }
    }

    pub fn finish(self) -> NonNull<PyObject> {
        unsafe {
            let this = ManuallyDrop::new(self);
            let bytes = compat::PyBytesWriter_FinishWithSize(this.writer, this.len as isize);
            NonNull::new_unchecked(bytes)
        }
    }

    #[inline]
    pub fn resize(&mut self, len: usize) {
        self.cap = len;
        unsafe {
            compat::PyBytesWriter_Resize(self.writer, len as isize);
            self.data = compat::PyBytesWriter_GetData(self.writer).cast::<u8>();
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self, len: usize) {
        let mut cap = self.cap;
        while len >= cap {
            if len < 262144 {
                cap *= 4;
            } else {
                cap *= 2;
            }
        }
        self.resize(cap);
    }

    fn insert_slices<const N: usize>(&mut self, bufs: [&[u8]; N]) {
        let len: usize = bufs.iter().map(|b| b.len()).sum();
        let new_len = self.len + len;
        if new_len > self.cap {
            self.grow(new_len);
        }
        let mut ptr = unsafe { self.data.add(self.len) };
        for buf in bufs {
            unsafe {
                std::ptr::copy_nonoverlapping(buf.as_ptr(), ptr, buf.len());
                ptr = ptr.add(buf.len());
            };
        }
        self.len = new_len;
    }
}

impl Drop for BytesWriter {
    fn drop(&mut self) {
        unsafe { compat::PyBytesWriter_Discard(self.writer) }
    }
}

impl std::io::Write for BytesWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.insert_slices([buf]);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        self.insert_slices([buf]);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

impl WriteSlices for BytesWriter {
    fn write_slices<const N: usize>(&mut self, bufs: [&[u8]; N]) -> Result<(), std::io::Error> {
        self.insert_slices(bufs);
        Ok(())
    }
}
