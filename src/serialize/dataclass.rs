// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
use crate::opt::*;
use crate::serialize::default::DefaultHook;
use crate::serialize::serializer::*;
use crate::state::State;
use crate::util::unlikely;

use serde::ser::{Serialize, SerializeMap, Serializer};

use smallvec::SmallVec;

#[inline]
fn has_slots(ob_type: *mut pyo3::ffi::PyTypeObject, state: *mut State) -> bool {
    unsafe {
        let tp_dict = (*ob_type).tp_dict;
        pyo3::ffi::PyDict_Contains(tp_dict, (*state).slots_str) == 1
    }
}

#[inline]
pub fn is_dataclass(ob_type: *mut pyo3::ffi::PyTypeObject, state: *mut State) -> bool {
    unsafe {
        let tp_dict = (*ob_type).tp_dict;
        !tp_dict.is_null()
            && pyo3::ffi::PyDict_Contains(tp_dict, (*state).dataclass_fields_str) == 1
    }
}

pub struct Dataclass<'a> {
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
    opts: Opt,
    default: &'a DefaultHook,
}

impl<'a> Dataclass<'a> {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        state: *mut State,
        opts: Opt,
        default: &'a DefaultHook,
    ) -> Self {
        Dataclass {
            ptr: ptr,
            state: state,
            opts: opts,
            default: default,
        }
    }
}

fn is_pseudo_field(field: *mut pyo3::ffi::PyObject, state: *mut State) -> bool {
    let field_type = unsafe { pyo3::ffi::PyObject_GetAttr(field, (*state).field_type_str) };
    unsafe { pyo3::ffi::Py_DECREF(field_type) };
    field_type.cast::<pyo3::ffi::PyTypeObject>() != unsafe { (*state).dataclass_field_type }
}

impl Serialize for Dataclass<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let fields =
            unsafe { pyo3::ffi::PyObject_GetAttr(self.ptr, (*self.state).dataclass_fields_str) };
        unsafe { pyo3::ffi::Py_DECREF(fields) };
        let len = unsafe { pydict_size(fields) } as usize;
        if unlikely(len == 0) {
            return serializer.serialize_map(Some(0))?.end();
        }

        let dict = {
            let ob_type = ob_type!(self.ptr);
            if has_slots(ob_type, self.state) {
                std::ptr::null_mut()
            } else {
                let dict = unsafe { pyo3::ffi::PyObject_GetAttr(self.ptr, (*self.state).dict_str) };
                unsafe { pyo3::ffi::Py_DECREF(dict) };
                dict
            }
        };

        let mut items: SmallVec<[(&str, *mut pyo3::ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);
        for (attr, field) in PyDictIter::from_pyobject(fields) {
            let key_as_str = unicode_to_str(attr.as_ptr()).map_err(serde::ser::Error::custom)?;
            if key_as_str.as_bytes()[0] == b'_' {
                continue;
            }

            if unlikely(dict.is_null()) {
                if !is_pseudo_field(field.as_ptr(), self.state) {
                    let value = unsafe { pyo3::ffi::PyObject_GetAttr(self.ptr, attr.as_ptr()) };
                    unsafe { pyo3::ffi::Py_DECREF(value) };
                    items.push((key_as_str, value));
                }
            } else {
                let value = unsafe { pyo3::ffi::PyDict_GetItem(dict, attr.as_ptr()) };
                if !value.is_null() {
                    items.push((key_as_str, value));
                } else if !is_pseudo_field(field.as_ptr(), self.state) {
                    let value = unsafe { pyo3::ffi::PyObject_GetAttr(self.ptr, attr.as_ptr()) };
                    unsafe { pyo3::ffi::Py_DECREF(value) };
                    items.push((key_as_str, value));
                }
            }
        }

        let mut map = serializer.serialize_map(Some(items.len()))?;
        for (key, value) in items.iter() {
            let pyvalue = PyObject::new(*value, self.state, self.opts, self.default);
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?
        }
        map.end()
    }
}
