// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::ffi::*;
use crate::opt::*;
use crate::serialize::default::DefaultHook;
use crate::serialize::serializer::*;
use crate::state::State;
use crate::util::unlikely;

use serde::ser::{Serialize, SerializeMap, Serializer};

use smallvec::SmallVec;

#[inline]
pub fn is_pydantic_model(ob_type: *mut pyo3::ffi::PyTypeObject, state: *mut State) -> bool {
    unsafe {
        let tp_dict = (*ob_type).tp_dict;
        !tp_dict.is_null()
            && (pyo3::ffi::PyDict_Contains(tp_dict, (*state).fields_str) == 1
                || pyo3::ffi::PyDict_Contains(tp_dict, (*state).pydantic_validator_str) == 1)
    }
}

pub struct PydanticModel<'a> {
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
    opts: Opt,
    default: &'a DefaultHook,
}

impl<'a> PydanticModel<'a> {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        state: *mut State,
        opts: Opt,
        default: &'a DefaultHook,
    ) -> Self {
        PydanticModel {
            ptr: ptr,
            state: state,
            opts: opts,
            default: default,
        }
    }
}

impl Serialize for PydanticModel<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let dict = unsafe { pyo3::ffi::PyObject_GetAttr(self.ptr, (*self.state).dict_str) };
        if unlikely(dict.is_null()) {
            unsafe { pyo3::ffi::PyErr_Clear() };
            return Err(serde::ser::Error::custom(
                "Pydantic model must have __dict__ attribute",
            ));
        }

        let extra_dict =
            unsafe { pyo3::ffi::PyObject_GetAttr(self.ptr, (*self.state).pydantic_extra_str) };
        if extra_dict.is_null() {
            unsafe { pyo3::ffi::PyErr_Clear() };
            let res = self.serialize_with_no_extra(serializer, dict);
            unsafe { pyo3::ffi::Py_DECREF(dict) };
            res
        } else {
            let res = if ob_type!(extra_dict) == &raw mut pyo3::ffi::PyDict_Type {
                self.serialize_with_extra(serializer, dict, extra_dict)
            } else {
                self.serialize_with_no_extra(serializer, dict)
            };
            unsafe {
                pyo3::ffi::Py_DECREF(dict);
                pyo3::ffi::Py_DECREF(extra_dict)
            };
            res
        }
    }
}

impl PydanticModel<'_> {
    fn serialize_with_no_extra<S>(
        &self,
        serializer: S,
        dict: *mut pyo3::ffi::PyObject,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = unsafe { pydict_size(dict) } as usize;
        if unlikely(len == 0) {
            return serializer.serialize_map(Some(0))?.end();
        }
        let mut items: SmallVec<[(&str, *mut pyo3::ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);
        for (key, value) in PyDictIter::from_pyobject(dict) {
            if unlikely(ob_type!(key.as_ptr()) != &raw mut pyo3::ffi::PyUnicode_Type) {
                return Err(serde::ser::Error::custom(KEY_MUST_BE_STR));
            }
            let key_as_str = unicode_to_str(key.as_ptr()).map_err(serde::ser::Error::custom)?;
            if unlikely(key_as_str.as_bytes()[0] == b'_') {
                continue;
            }
            items.push((key_as_str, value.as_ptr()));
        }

        if self.opts & SORT_KEYS != 0 {
            items.sort_unstable_by(|a, b| a.0.cmp(b.0));
        }

        let mut map = serializer.serialize_map(Some(items.len()))?;
        for (key, value) in items.iter() {
            let pyvalue = PyObject::new(*value, self.state, self.opts, self.default);
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }

    fn serialize_with_extra<S>(
        &self,
        serializer: S,
        dict: *mut pyo3::ffi::PyObject,
        extra_dict: *mut pyo3::ffi::PyObject,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let iter = PyDictIter::from_pyobject(dict).chain(PyDictIter::from_pyobject(extra_dict));
        let len = iter.size_hint().0;
        if unlikely(len == 0) {
            return serializer.serialize_map(Some(0))?.end();
        }
        let mut items: SmallVec<[(&str, *mut pyo3::ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);
        for (key, value) in iter {
            if unlikely(ob_type!(key.as_ptr()) != &raw mut pyo3::ffi::PyUnicode_Type) {
                return Err(serde::ser::Error::custom(KEY_MUST_BE_STR));
            }
            let key_as_str = unicode_to_str(key.as_ptr()).map_err(serde::ser::Error::custom)?;
            if unlikely(key_as_str.as_bytes()[0] == b'_') {
                continue;
            }
            items.push((key_as_str, value.as_ptr()));
        }

        if self.opts & SORT_KEYS != 0 {
            items.sort_unstable_by(|a, b| a.0.cmp(b.0));
        }

        let mut map = serializer.serialize_map(Some(items.len()))?;
        for (key, value) in items.iter() {
            let pyvalue = PyObject::new(*value, self.state, self.opts, self.default);
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}
