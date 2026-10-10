use crate::msgpack;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::ffi::*;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyTuple};
use std::os::raw::{c_int, c_uint, c_void};
use std::ptr::null_mut;

#[repr(C)]
pub struct PyFragment {
    pub ob_base: PyObject,
    pub data: *mut PyObject,
}

#[no_mangle]
unsafe extern "C" fn fragment_new(
    subtype: *mut PyTypeObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    let py = Python::assume_attached();
    let args = Borrowed::from_ptr(py, args).cast_unchecked::<PyTuple>();
    let kwds = Borrowed::from_ptr_or_opt(py, kwds).map(|v| v.cast_unchecked::<PyDict>());
    if args.len() != 1 || kwds.is_some_and(|v| v.len() != 0) {
        PyTypeError::new_err("Fragment.__new__() takes 1 positional argument").restore(py);
        return null_mut();
    }
    let data = args.get_item(0).unwrap();
    if !data.is_instance_of::<PyBytes>() {
        PyTypeError::new_err("Fragment.__new__() first argument must be bytes").restore(py);
        return null_mut();
    }
    let contents = data.cast_unchecked::<PyBytes>().as_bytes();
    let mut validator = msgpack::Validator::new(contents);
    match validator.validate() {
        Ok(()) => (),
        Err(err) => {
            PyValueError::new_err(err.to_string()).restore(py);
            return null_mut();
        }
    }
    let op = (*subtype).tp_alloc.unwrap()(subtype, 0);
    let obj = &mut *op.cast::<PyFragment>();
    obj.data = data.into_ptr();
    op
}

#[no_mangle]
unsafe extern "C" fn fragment_dealloc(op: *mut PyObject) {
    let ob_type = Py_TYPE(op);
    Py_DECREF((*op.cast::<PyFragment>()).data);
    (*ob_type).tp_free.unwrap()(op.cast::<c_void>());
    Py_DECREF(ob_type.cast::<PyObject>());
}

pub fn create_fragment_type<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    let mut slots: [PyType_Slot; 3] = [
        PyType_Slot {
            slot: Py_tp_new,
            pfunc: fragment_new as *mut c_void,
        },
        PyType_Slot {
            slot: Py_tp_dealloc,
            pfunc: fragment_dealloc as *mut c_void,
        },
        PyType_Slot {
            slot: 0,
            pfunc: null_mut(),
        },
    ];
    let mut spec = PyType_Spec {
        name: c"ormsgpack.Fragment".as_ptr(),
        basicsize: std::mem::size_of::<PyFragment>() as c_int,
        itemsize: 0,
        flags: Py_TPFLAGS_DEFAULT as c_uint,
        slots: slots.as_mut_ptr(),
    };
    unsafe { Bound::from_owned_ptr_or_err(py, PyType_FromSpec(&mut spec)) }
}
