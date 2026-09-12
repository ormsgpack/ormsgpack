// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
use ahash::RandomState;
use pyo3::prelude::*;
use pyo3::types::PyString;
use simdutf8::basic::{from_utf8, Utf8Error};
#[cfg(not(Py_GIL_DISABLED))]
use std::cell::RefCell;
use std::hash::BuildHasher;
use std::hash::Hasher;
#[cfg(Py_GIL_DISABLED)]
use std::sync::Mutex;

pub struct KeyMap<const C: usize> {
    #[cfg(Py_GIL_DISABLED)]
    entries: Mutex<Vec<Option<Py<PyString>>>>,
    #[cfg(not(Py_GIL_DISABLED))]
    entries: RefCell<Vec<Option<Py<PyString>>>>,
    hash_builder: RandomState,
}

// SAFETY: On GIL builds, every access to entries requires a Python token and
// therefore holds the GIL. RefCell handles reentrant access on the same thread.
#[cfg(not(Py_GIL_DISABLED))]
unsafe impl<const C: usize> Sync for KeyMap<C> {}

impl<const C: usize> KeyMap<C> {
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(C);
        for _ in 0..C {
            entries.push(None);
        }
        KeyMap {
            #[cfg(Py_GIL_DISABLED)]
            entries: Mutex::new(entries),
            #[cfg(not(Py_GIL_DISABLED))]
            entries: RefCell::new(entries),
            hash_builder: RandomState::new(),
        }
    }

    pub fn get<'py>(&self, py: Python<'py>, key: &[u8]) -> Result<Bound<'py, PyString>, Utf8Error> {
        let hash = {
            let mut hasher = self.hash_builder.build_hasher();
            hasher.write(key);
            hasher.finish()
        } as usize;
        let index = hash % C;
        #[cfg(Py_GIL_DISABLED)]
        let mut entries = self.entries.lock().unwrap();
        #[cfg(not(Py_GIL_DISABLED))]
        let mut entries = self.entries.borrow_mut();
        let entry = match &mut entries[index] {
            Some(v) if unicode_to_str(v.bind_borrowed(py)).unwrap().as_bytes() == key => v,
            _ => {
                let pykey = unicode_from_str(py, from_utf8(key)?);
                hash_str(pykey.as_borrowed());
                entries[index] = Some(pykey.unbind());
                match &mut entries[index] {
                    Some(v) => v,
                    _ => unreachable!(),
                }
            }
        };
        Ok(entry.bind_borrowed(py).to_owned())
    }
}
