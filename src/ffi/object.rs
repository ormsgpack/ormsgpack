// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::ffi::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyInt, PyList, PyString, PyType};
use pyo3::PyTypeInfo;
use std::ffi::CStr;

#[inline(always)]
pub fn hasattr<T>(obj: Borrowed<'_, '_, T>, name: Borrowed<'_, '_, PyString>) -> PyResult<bool> {
    #[cfg(Py_3_13)]
    {
        match unsafe { PyObject_HasAttrWithError(obj.as_ptr(), name.as_ptr()) } {
            1 => Ok(true),
            0 => Ok(false),
            _ => Err(PyErr::fetch(obj.py())),
        }
    }
    #[cfg(not(Py_3_13))]
    {
        Ok(unsafe { PyObject_HasAttr(obj.as_ptr(), name.as_ptr()) } == 1)
    }
}

#[inline(always)]
pub fn get_type<'a, 'py, T>(obj: Borrowed<'a, 'py, T>) -> Borrowed<'a, 'py, PyType> {
    unsafe { Borrowed::from_ptr(obj.py(), Py_TYPE(obj.as_ptr()).cast()).cast_unchecked::<PyType>() }
}

#[inline(always)]
pub fn get_type_dict<'a, 'py>(obj: Borrowed<'a, 'py, PyType>) -> Option<Borrowed<'a, 'py, PyDict>> {
    unsafe {
        let tp_dict = (*obj.as_type_ptr()).tp_dict;
        if tp_dict.is_null() {
            None
        } else {
            Some(Borrowed::from_ptr(obj.py(), tp_dict).cast_unchecked::<PyDict>())
        }
    }
}

#[cold]
pub fn get_type_name<T>(obj: Borrowed<'_, '_, T>) -> String {
    let type_ptr = get_type(obj).as_type_ptr();
    unsafe {
        CStr::from_ptr((*type_ptr).tp_name)
            .to_string_lossy()
            .into_owned()
    }
}

#[inline(always)]
pub fn cast_exact<'a, 'py, T: PyTypeInfo>(
    value: Borrowed<'a, 'py, PyAny>,
) -> Option<Borrowed<'a, 'py, T>> {
    if get_type(value).as_type_ptr() == T::type_object_raw(value.py()) {
        Some(unsafe { value.cast_unchecked() })
    } else {
        None
    }
}

#[inline(always)]
pub fn cast_into_exact<T: PyTypeInfo>(value: Bound<'_, PyAny>) -> Option<Bound<'_, T>> {
    if get_type(value.as_borrowed()).as_type_ptr() == T::type_object_raw(value.py()) {
        Some(unsafe { value.cast_into_unchecked() })
    } else {
        None
    }
}

pub trait TypeCheck: PyTypeInfo {
    #[inline(always)]
    fn matches(type_obj: Borrowed<'_, '_, PyType>) -> bool {
        let type_ptr = type_obj.as_type_ptr();
        let self_type_ptr = Self::type_object_raw(type_obj.py());
        type_ptr == self_type_ptr || unsafe { PyType_IsSubtype(type_ptr, self_type_ptr) != 0 }
    }
}

impl TypeCheck for PyDict {
    #[inline(always)]
    fn matches(type_obj: Borrowed<'_, '_, PyType>) -> bool {
        unsafe { PyType_HasFeature(type_obj.as_type_ptr(), Py_TPFLAGS_DICT_SUBCLASS) != 0 }
    }
}

impl TypeCheck for PyInt {
    #[inline(always)]
    fn matches(type_obj: Borrowed<'_, '_, PyType>) -> bool {
        unsafe { PyType_HasFeature(type_obj.as_type_ptr(), Py_TPFLAGS_LONG_SUBCLASS) != 0 }
    }
}

impl TypeCheck for PyList {
    #[inline(always)]
    fn matches(type_obj: Borrowed<'_, '_, PyType>) -> bool {
        unsafe { PyType_HasFeature(type_obj.as_type_ptr(), Py_TPFLAGS_LIST_SUBCLASS) != 0 }
    }
}

impl TypeCheck for PyString {
    #[inline(always)]
    fn matches(type_obj: Borrowed<'_, '_, PyType>) -> bool {
        unsafe { PyType_HasFeature(type_obj.as_type_ptr(), Py_TPFLAGS_UNICODE_SUBCLASS) != 0 }
    }
}

#[derive(Clone, Copy)]
pub struct BorrowedWithType<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
    type_obj: Borrowed<'a, 'py, PyType>,
}

impl<'a, 'py> BorrowedWithType<'a, 'py> {
    #[inline(always)]
    pub fn new(obj: Borrowed<'a, 'py, PyAny>) -> Self {
        Self {
            obj,
            type_obj: get_type(obj),
        }
    }

    #[inline(always)]
    pub fn py(self) -> Python<'py> {
        self.obj.py()
    }

    #[inline(always)]
    pub fn as_borrowed(self) -> Borrowed<'a, 'py, PyAny> {
        self.obj
    }

    #[inline(always)]
    pub fn cast_exact<T: PyTypeInfo>(self) -> Option<Borrowed<'a, 'py, T>> {
        if self.get_type_ptr() == T::type_object_raw(self.obj.py()) {
            Some(unsafe { self.obj.cast_unchecked() })
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn cast<T: TypeCheck>(self) -> Option<Borrowed<'a, 'py, T>> {
        if T::matches(self.type_obj) {
            Some(unsafe { self.obj.cast_unchecked() })
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_type(self) -> Borrowed<'a, 'py, PyType> {
        self.type_obj
    }

    #[inline(always)]
    pub fn get_type_ptr(self) -> *mut PyTypeObject {
        self.type_obj.as_type_ptr()
    }
}
