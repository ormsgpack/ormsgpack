// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::ffi::*;
use std::os::raw::c_int;

mod int;
mod unicode;

pub use unicode::*;

#[inline(always)]
pub unsafe fn pybytes_as_mut_u8(op: *mut PyObject) -> *mut u8 {
    PyBytes_AsString(op).cast::<u8>()
}

#[inline(always)]
pub unsafe fn pydict_size(mp: *mut PyObject) -> Py_ssize_t {
    PyDict_Size(mp)
}

#[inline(always)]
pub unsafe fn pyobject_call_one_arg(func: *mut PyObject, arg: *mut PyObject) -> *mut PyObject {
    PyObject_CallFunctionObjArgs(func, arg, std::ptr::null_mut::<PyObject>())
}

#[inline(always)]
pub unsafe fn pyobject_call_method_no_args(
    self_: *mut PyObject,
    name: *mut PyObject,
) -> *mut PyObject {
    PyObject_CallMethodObjArgs(self_, name, std::ptr::null_mut::<PyObject>())
}

#[inline(always)]
pub unsafe fn pyobject_call_method_one_arg(
    self_: *mut PyObject,
    name: *mut PyObject,
    arg: *mut PyObject,
) -> *mut PyObject {
    PyObject_CallMethodObjArgs(self_, name, arg, std::ptr::null_mut::<PyObject>())
}

#[inline(always)]
pub unsafe fn pytuple_get_item(op: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
    PyTuple_GetItem(op, i)
}

#[inline(always)]
pub unsafe fn pytuple_set_item(op: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) {
    if PyTuple_SetItem(op, i, v) == -1 {
        unreachable!();
    }
}
