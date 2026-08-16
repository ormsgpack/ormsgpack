// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::Opt;
use crate::serialize::default::DefaultHook;
use crate::serialize::serializer::{DictKey, PyObject as ObjectSerializer};
use crate::state::State;

use pyo3::ffi::*;
use serde::ser::{Serialize, Serializer};

pub struct Enum<'a> {
    ptr: *mut PyObject,
    state: *mut State,
    opts: Opt,
    default: &'a DefaultHook,
}

impl<'a> Enum<'a> {
    pub fn new(ptr: *mut PyObject, state: *mut State, opts: Opt, default: &'a DefaultHook) -> Self {
        Self {
            ptr: ptr,
            state: state,
            opts: opts,
            default: default,
        }
    }
}

impl Serialize for Enum<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = unsafe { PyObject_GetAttr(self.ptr, (*self.state).value_str) };
        let result =
            ObjectSerializer::new(value, self.state, self.opts, self.default).serialize(serializer);
        unsafe { Py_DECREF(value) };
        result
    }
}

pub struct EnumDictKey {
    ptr: *mut PyObject,
    state: *mut State,
    opts: Opt,
}

impl EnumDictKey {
    pub fn new(ptr: *mut PyObject, state: *mut State, opts: Opt) -> Self {
        Self {
            ptr: ptr,
            state: state,
            opts: opts,
        }
    }
}

impl Serialize for EnumDictKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = unsafe { PyObject_GetAttr(self.ptr, (*self.state).value_str) };
        let result = DictKey::new(value, self.state, self.opts).serialize(serializer);
        unsafe { Py_DECREF(value) };
        result
    }
}
