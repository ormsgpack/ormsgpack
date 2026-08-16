// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
use crate::opt::*;
use crate::serialize::default::DefaultHook;
use crate::serialize::serializer::*;
use crate::state::State;

use serde::ser::{Serialize, SerializeSeq, Serializer};

pub struct Tuple<'a> {
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
    opts: Opt,
    default: &'a DefaultHook,
}

impl<'a> Tuple<'a> {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        state: *mut State,
        opts: Opt,
        default: &'a DefaultHook,
    ) -> Self {
        Tuple {
            ptr: ptr,
            state: state,
            opts: opts,
            default: default,
        }
    }
}

impl Serialize for Tuple<'_> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = unsafe { pyo3::ffi::Py_SIZE(self.ptr) } as usize;
        let mut seq = serializer.serialize_seq(Some(len))?;
        for i in 0..len {
            let item = unsafe { pytuple_get_item(self.ptr, i as isize) };
            let value = PyObject::new(item, self.state, self.opts, self.default);
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

pub struct TupleDictKey {
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
    opts: Opt,
}

impl TupleDictKey {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, state: *mut State, opts: Opt) -> Self {
        TupleDictKey {
            ptr: ptr,
            state: state,
            opts: opts,
        }
    }
}

impl Serialize for TupleDictKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = unsafe { pyo3::ffi::Py_SIZE(self.ptr) } as usize;
        let mut seq = serializer.serialize_seq(Some(len))?;
        for i in 0..len {
            let item = unsafe { pytuple_get_item(self.ptr, i as isize) };
            let value = DictKey::new(item, self.state, self.opts);
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}
