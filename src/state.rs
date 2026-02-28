// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::KeyMap;
use crate::ext::create_ext_type;
use crate::fragment::create_fragment_type;
use pyo3::ffi::*;
use std::ffi::CStr;
use std::ptr::null_mut;
use std::sync::OnceLock;

pub struct NumpyTypes {
    pub array: *mut PyTypeObject,
    pub float64: *mut PyTypeObject,
    pub float32: *mut PyTypeObject,
    pub float16: *mut PyTypeObject,
    pub int64: *mut PyTypeObject,
    pub int32: *mut PyTypeObject,
    pub int16: *mut PyTypeObject,
    pub int8: *mut PyTypeObject,
    pub uint64: *mut PyTypeObject,
    pub uint32: *mut PyTypeObject,
    pub uint16: *mut PyTypeObject,
    pub uint8: *mut PyTypeObject,
    pub bool_: *mut PyTypeObject,
    pub datetime64: *mut PyTypeObject,
}

#[inline]
unsafe fn get_type(module_dict: *mut PyObject, type_name: &CStr) -> *mut PyTypeObject {
    PyMapping_GetItemString(module_dict, type_name.as_ptr()).cast::<PyTypeObject>()
}

#[cold]
unsafe fn load_type(module_name: &CStr, type_name: &CStr) -> *mut PyTypeObject {
    let module = PyImport_ImportModule(module_name.as_ptr());
    let module_dict = PyObject_GenericGetDict(module, null_mut());
    let ptr = get_type(module_dict, type_name);
    Py_DECREF(module_dict);
    Py_DECREF(module);
    ptr
}

#[cold]
fn load_numpy_types() -> Option<NumpyTypes> {
    unsafe {
        let numpy = PyImport_ImportModule(c"numpy".as_ptr());
        if numpy.is_null() {
            PyErr_Clear();
            return None;
        }

        let numpy_dict = PyObject_GenericGetDict(numpy, null_mut());
        let types = NumpyTypes {
            array: get_type(numpy_dict, c"ndarray"),
            float16: get_type(numpy_dict, c"half"),
            float32: get_type(numpy_dict, c"float32"),
            float64: get_type(numpy_dict, c"float64"),
            int8: get_type(numpy_dict, c"int8"),
            int16: get_type(numpy_dict, c"int16"),
            int32: get_type(numpy_dict, c"int32"),
            int64: get_type(numpy_dict, c"int64"),
            uint16: get_type(numpy_dict, c"uint16"),
            uint32: get_type(numpy_dict, c"uint32"),
            uint64: get_type(numpy_dict, c"uint64"),
            uint8: get_type(numpy_dict, c"uint8"),
            bool_: get_type(numpy_dict, c"bool_"),
            datetime64: get_type(numpy_dict, c"datetime64"),
        };
        Py_DECREF(numpy_dict);
        Py_DECREF(numpy);
        Some(types)
    }
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct State {
    numpy_types: OnceLock<Option<NumpyTypes>>,
    pub dataclass_field_type: *mut PyTypeObject,
    pub enum_type: *mut PyTypeObject,
    pub ext_type: *mut PyTypeObject,
    pub fragment_type: *mut PyTypeObject,
    pub uuid_type: *mut PyTypeObject,
    pub array_struct_str: *mut PyObject,
    pub dataclass_fields_str: *mut PyObject,
    pub default_str: *mut PyObject,
    pub descr_str: *mut PyObject,
    pub dict_str: *mut PyObject,
    pub dtype_str: *mut PyObject,
    pub ext_hook_str: *mut PyObject,
    pub field_type_str: *mut PyObject,
    pub fields_str: *mut PyObject,
    pub int_str: *mut PyObject,
    pub normalize_str: *mut PyObject,
    pub option_str: *mut PyObject,
    pub pydantic_extra_str: *mut PyObject,
    pub pydantic_validator_str: *mut PyObject,
    pub slots_str: *mut PyObject,
    pub utcoffset_str: *mut PyObject,
    pub value_str: *mut PyObject,
    pub MsgpackEncodeError: *mut PyObject,
    pub MsgpackDecodeError: *mut PyObject,
    pub key_map: KeyMap<512>,
}

impl State {
    #[cold]
    pub fn new() -> Self {
        unsafe {
            Self {
                numpy_types: OnceLock::new(),
                dataclass_field_type: load_type(c"dataclasses", c"_FIELD"),
                enum_type: load_type(c"enum", c"EnumMeta"),
                ext_type: create_ext_type(),
                fragment_type: create_fragment_type(),
                uuid_type: load_type(c"uuid", c"UUID"),
                array_struct_str: PyUnicode_InternFromString(c"__array_struct__".as_ptr()),
                dataclass_fields_str: PyUnicode_InternFromString(c"__dataclass_fields__".as_ptr()),
                default_str: PyUnicode_InternFromString(c"default".as_ptr()),
                descr_str: PyUnicode_InternFromString(c"descr".as_ptr()),
                dict_str: PyUnicode_InternFromString(c"__dict__".as_ptr()),
                dtype_str: PyUnicode_InternFromString(c"dtype".as_ptr()),
                ext_hook_str: PyUnicode_InternFromString(c"ext_hook".as_ptr()),
                field_type_str: PyUnicode_InternFromString(c"_field_type".as_ptr()),
                fields_str: PyUnicode_InternFromString(c"__fields__".as_ptr()),
                int_str: PyUnicode_InternFromString(c"int".as_ptr()),
                normalize_str: PyUnicode_InternFromString(c"normalize".as_ptr()),
                option_str: PyUnicode_InternFromString(c"option".as_ptr()),
                pydantic_extra_str: PyUnicode_InternFromString(c"__pydantic_extra__".as_ptr()),
                pydantic_validator_str: PyUnicode_InternFromString(
                    c"__pydantic_validator__".as_ptr(),
                ),
                slots_str: PyUnicode_InternFromString(c"__slots__".as_ptr()),
                utcoffset_str: PyUnicode_InternFromString(c"utcoffset".as_ptr()),
                value_str: PyUnicode_InternFromString(c"value".as_ptr()),
                MsgpackEncodeError: Py_NewRef(PyExc_TypeError),
                MsgpackDecodeError: Py_NewRef(PyExc_ValueError),
                key_map: KeyMap::new(),
            }
        }
    }

    pub fn get_numpy_types(&self) -> &Option<NumpyTypes> {
        self.numpy_types.get_or_init(load_numpy_types)
    }
}
