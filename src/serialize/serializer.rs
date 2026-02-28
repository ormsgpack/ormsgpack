// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
use crate::msgpack;
use crate::opt::*;
use crate::serialize::bytearray::*;
use crate::serialize::bytes::*;
use crate::serialize::dataclass::*;
use crate::serialize::datetime::*;
use crate::serialize::default::*;
use crate::serialize::dict::*;
use crate::serialize::ext::*;
use crate::serialize::fragment::*;
use crate::serialize::list::*;
use crate::serialize::memoryview::*;
use crate::serialize::numpy::*;
use crate::serialize::pydantic::*;
use crate::serialize::str::*;
use crate::serialize::tuple::*;
use crate::serialize::uuid::*;
use crate::serialize::writer::*;
use crate::state::State;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::os::raw::c_ulong;
use std::ptr::NonNull;

pub fn serialize(
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    opts: Opt,
) -> Result<NonNull<pyo3::ffi::PyObject>, String> {
    let mut buf = BytesWriter::default();
    let default_hook = DefaultHook::new(default);
    let obj = PyObject::new(ptr, state, opts, &default_hook);
    let mut ser = msgpack::Serializer::new(&mut buf);
    let res = obj.serialize(&mut ser);
    match res {
        Ok(_) => Ok(buf.finish()),
        Err(err) => {
            unsafe { pyo3::ffi::Py_DECREF(buf.finish().as_ptr()) };
            Err(err.to_string())
        }
    }
}

#[inline(always)]
fn is_subclass(op: *mut pyo3::ffi::PyTypeObject, feature: c_ulong) -> bool {
    unsafe { pyo3::ffi::PyType_HasFeature(op, feature) != 0 }
}

pub struct PyObject<'a> {
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
    opts: Opt,
    default: &'a DefaultHook,
}

impl<'a> PyObject<'a> {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        state: *mut State,
        opts: Opt,
        default: &'a DefaultHook,
    ) -> Self {
        PyObject {
            ptr: ptr,
            state: state,
            opts: opts,
            default: default,
        }
    }

    fn serialize_with_default_hook<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj = self
            .default
            .enter_call(self.ptr)
            .map_err(serde::ser::Error::custom)?;
        let res = PyObject::new(obj, self.state, self.opts, self.default).serialize(serializer);
        self.default.leave_call();
        unsafe { pyo3::ffi::Py_DECREF(obj) };
        res
    }

    #[inline(never)]
    fn serialize_unlikely<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ob_type = ob_type!(self.ptr);

        if self.opts & PASSTHROUGH_DATETIME == 0 {
            let datetime_api = unsafe { *pyo3::ffi::PyDateTimeAPI() };
            if ob_type == datetime_api.DateTimeType {
                match DateTime::new(self.ptr, self.state, self.opts) {
                    Ok(val) => return val.serialize(serializer),
                    Err(err) => return Err(serde::ser::Error::custom(err)),
                }
            }
            if ob_type == datetime_api.DateType {
                return Date::new(self.ptr).serialize(serializer);
            }
            if ob_type == datetime_api.TimeType {
                match Time::new(self.ptr, self.opts) {
                    Ok(val) => return val.serialize(serializer),
                    Err(err) => return Err(serde::ser::Error::custom(err)),
                };
            }
        }

        if self.opts & PASSTHROUGH_TUPLE == 0 && ob_type == &raw mut pyo3::ffi::PyTuple_Type {
            return Tuple::new(self.ptr, self.state, self.opts, self.default).serialize(serializer);
        }

        if self.opts & PASSTHROUGH_UUID == 0 && ob_type == unsafe { (*self.state).uuid_type } {
            return UUID::new(self.ptr, self.state).serialize(serializer);
        }

        if ob_type!(ob_type) == unsafe { (*self.state).enum_type } {
            if self.opts & PASSTHROUGH_ENUM == 0 {
                let value =
                    unsafe { pyo3::ffi::PyObject_GetAttr(self.ptr, (*self.state).value_str) };
                unsafe { pyo3::ffi::Py_DECREF(value) };
                return PyObject::new(value, self.state, self.opts, self.default)
                    .serialize(serializer);
            } else {
                return self.serialize_with_default_hook(serializer);
            }
        }

        if self.opts & PASSTHROUGH_SUBCLASS == 0 {
            if is_subclass(ob_type, pyo3::ffi::Py_TPFLAGS_UNICODE_SUBCLASS) {
                return StrSubclass::new(self.ptr, self.opts).serialize(serializer);
            }
            if is_subclass(ob_type, pyo3::ffi::Py_TPFLAGS_LONG_SUBCLASS) {
                match Int::new(self.ptr) {
                    Ok(val) => return val.serialize(serializer),
                    Err(err) => {
                        if self.opts & PASSTHROUGH_BIG_INT != 0 {
                            return self.serialize_with_default_hook(serializer);
                        } else {
                            return Err(serde::ser::Error::custom(err));
                        }
                    }
                }
            }
            if is_subclass(ob_type, pyo3::ffi::Py_TPFLAGS_LIST_SUBCLASS) {
                return List::new(self.ptr, self.state, self.opts, self.default)
                    .serialize(serializer);
            }
            if is_subclass(ob_type, pyo3::ffi::Py_TPFLAGS_DICT_SUBCLASS) {
                return Dict::new(self.ptr, self.state, self.opts, self.default)
                    .serialize(serializer);
            }
        }

        if ob_type == unsafe { (*self.state).ext_type } {
            return Ext::new(self.ptr).serialize(serializer);
        }

        if self.opts & PASSTHROUGH_DATACLASS == 0 && is_dataclass(ob_type, self.state) {
            return Dataclass::new(self.ptr, self.state, self.opts, self.default)
                .serialize(serializer);
        }

        if self.opts & SERIALIZE_PYDANTIC != 0 && is_pydantic_model(ob_type, self.state) {
            return PydanticModel::new(self.ptr, self.state, self.opts, self.default)
                .serialize(serializer);
        }

        if self.opts & SERIALIZE_NUMPY != 0 {
            if let Some(numpy_types_ref) = unsafe { (*self.state).get_numpy_types() } {
                if ob_type == numpy_types_ref.bool_ {
                    return NumpyBool::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.datetime64 {
                    return NumpyDatetime64::new(self.ptr, self.state, self.opts)
                        .serialize(serializer);
                }
                if ob_type == numpy_types_ref.float16 {
                    return NumpyFloat16::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.float32 {
                    return NumpyFloat32::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.float64 {
                    return NumpyFloat64::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.int8 {
                    return NumpyInt8::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.int16 {
                    return NumpyInt16::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.int32 {
                    return NumpyInt32::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.int64 {
                    return NumpyInt64::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.uint8 {
                    return NumpyUint8::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.uint16 {
                    return NumpyUint16::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.uint32 {
                    return NumpyUint32::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.uint64 {
                    return NumpyUint64::new(self.ptr).serialize(serializer);
                }
                if ob_type == numpy_types_ref.array {
                    match NumpyArray::new(self.ptr, self.state, self.opts) {
                        Ok(val) => return val.serialize(serializer),
                        Err(PyArrayError::Malformed) => {
                            return Err(serde::ser::Error::custom("numpy array is malformed"))
                        }
                        Err(PyArrayError::NotContiguous)
                        | Err(PyArrayError::UnsupportedDataType) => {
                            if self.default.inner.is_none() {
                                return Err(serde::ser::Error::custom("numpy array is not C contiguous; use ndarray.tolist() in default"));
                            }
                        }
                    }
                }
            }
        }

        if ob_type == &raw mut pyo3::ffi::PyByteArray_Type {
            return ByteArray::new(self.ptr).serialize(serializer);
        }

        if ob_type == &raw mut pyo3::ffi::PyMemoryView_Type {
            return MemoryView::new(self.ptr).serialize(serializer);
        }

        if ob_type == unsafe { (*self.state).fragment_type } {
            return Fragment::new(self.ptr).serialize(serializer);
        }

        self.serialize_with_default_hook(serializer)
    }
}

impl Serialize for PyObject<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ob_type = ob_type!(self.ptr);
        if ob_type == &raw mut pyo3::ffi::PyUnicode_Type {
            Str::new(self.ptr, self.opts).serialize(serializer)
        } else if ob_type == &raw mut pyo3::ffi::PyBytes_Type {
            Bytes::new(self.ptr).serialize(serializer)
        } else if ob_type == &raw mut pyo3::ffi::PyLong_Type {
            match Int::new(self.ptr) {
                Ok(val) => val.serialize(serializer),
                Err(err) => {
                    if self.opts & PASSTHROUGH_BIG_INT != 0 {
                        self.serialize_with_default_hook(serializer)
                    } else {
                        Err(serde::ser::Error::custom(err))
                    }
                }
            }
        } else if ob_type == &raw mut pyo3::ffi::PyBool_Type {
            serializer.serialize_bool(unsafe { self.ptr == pyo3::ffi::Py_True() })
        } else if self.ptr == unsafe { pyo3::ffi::Py_None() } {
            serializer.serialize_unit()
        } else if ob_type == &raw mut pyo3::ffi::PyFloat_Type {
            serializer.serialize_f64(unsafe { pyo3::ffi::PyFloat_AS_DOUBLE(self.ptr) })
        } else if ob_type == &raw mut pyo3::ffi::PyList_Type {
            List::new(self.ptr, self.state, self.opts, self.default).serialize(serializer)
        } else if ob_type == &raw mut pyo3::ffi::PyDict_Type {
            Dict::new(self.ptr, self.state, self.opts, self.default).serialize(serializer)
        } else {
            self.serialize_unlikely(serializer)
        }
    }
}

pub struct DictTupleKey {
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
    opts: Opt,
}

impl DictTupleKey {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, state: *mut State, opts: Opt) -> Self {
        DictTupleKey {
            ptr: ptr,
            state: state,
            opts: opts,
        }
    }
}

impl Serialize for DictTupleKey {
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

pub struct DictKey {
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
    opts: Opt,
}

impl DictKey {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, state: *mut State, opts: Opt) -> Self {
        DictKey {
            ptr: ptr,
            state: state,
            opts: opts,
        }
    }

    #[inline(never)]
    fn serialize_unlikely<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ob_type = ob_type!(self.ptr);

        let datetime_api = unsafe { *pyo3::ffi::PyDateTimeAPI() };
        if ob_type == datetime_api.DateTimeType {
            match DateTime::new(self.ptr, self.state, self.opts) {
                Ok(val) => return val.serialize(serializer),
                Err(err) => return Err(serde::ser::Error::custom(err)),
            }
        }
        if ob_type == datetime_api.DateType {
            return Date::new(self.ptr).serialize(serializer);
        }
        if ob_type == datetime_api.TimeType {
            match Time::new(self.ptr, self.opts) {
                Ok(val) => return val.serialize(serializer),
                Err(err) => return Err(serde::ser::Error::custom(err)),
            };
        }

        if ob_type == &raw mut pyo3::ffi::PyTuple_Type {
            return DictTupleKey::new(self.ptr, self.state, self.opts).serialize(serializer);
        }

        if ob_type == unsafe { (*self.state).uuid_type } {
            return UUID::new(self.ptr, self.state).serialize(serializer);
        }

        if ob_type!(ob_type) == unsafe { (*self.state).enum_type } {
            let value = unsafe { pyo3::ffi::PyObject_GetAttr(self.ptr, (*self.state).value_str) };
            unsafe { pyo3::ffi::Py_DECREF(value) };
            return DictKey::new(value, self.state, self.opts).serialize(serializer);
        }

        if is_subclass(ob_type, pyo3::ffi::Py_TPFLAGS_UNICODE_SUBCLASS) {
            return StrSubclass::new(self.ptr, self.opts).serialize(serializer);
        }
        if is_subclass(ob_type, pyo3::ffi::Py_TPFLAGS_LONG_SUBCLASS) {
            match Int::new(self.ptr) {
                Ok(val) => return val.serialize(serializer),
                Err(err) => return Err(serde::ser::Error::custom(err)),
            }
        }

        if ob_type == &raw mut pyo3::ffi::PyMemoryView_Type {
            return MemoryView::new(self.ptr).serialize(serializer);
        }

        Err(serde::ser::Error::custom(
            "Dict key must a type serializable with OPT_NON_STR_KEYS",
        ))
    }
}

impl Serialize for DictKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ob_type = ob_type!(self.ptr);
        if ob_type == &raw mut pyo3::ffi::PyUnicode_Type {
            Str::new(self.ptr, self.opts).serialize(serializer)
        } else if ob_type == &raw mut pyo3::ffi::PyBytes_Type {
            Bytes::new(self.ptr).serialize(serializer)
        } else if ob_type == &raw mut pyo3::ffi::PyLong_Type {
            match Int::new(self.ptr) {
                Ok(val) => val.serialize(serializer),
                Err(err) => Err(serde::ser::Error::custom(err)),
            }
        } else if ob_type == &raw mut pyo3::ffi::PyBool_Type {
            serializer.serialize_bool(unsafe { self.ptr == pyo3::ffi::Py_True() })
        } else if self.ptr == unsafe { pyo3::ffi::Py_None() } {
            serializer.serialize_unit()
        } else if ob_type == &raw mut pyo3::ffi::PyFloat_Type {
            serializer.serialize_f64(unsafe { pyo3::ffi::PyFloat_AS_DOUBLE(self.ptr) })
        } else {
            self.serialize_unlikely(serializer)
        }
    }
}
