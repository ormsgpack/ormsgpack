use crate::ffi::*;
use crate::msgpack;
use pyo3::ffi::*;
use std::os::raw::{c_char, c_int, c_uint, c_void};
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
    if Py_SIZE(args) != 1 || (!kwds.is_null() && pydict_size(kwds) != 0) {
        PyErr_SetString(
            PyExc_TypeError,
            c"Fragment.__new__() takes 1 positional argument".as_ptr(),
        );
        return null_mut();
    }
    let data = pytuple_get_item(args, 0);
    if PyBytes_Check(data) == 0 {
        PyErr_SetString(
            PyExc_TypeError,
            c"Fragment.__new__() first argument must be bytes".as_ptr(),
        );
        return null_mut();
    }
    let contents = pybytes_as_bytes(data);
    let mut validator = msgpack::Validator::new(contents);
    match validator.validate() {
        Ok(()) => (),
        Err(err) => {
            PyErr_SetString(PyExc_ValueError, err.to_string().as_ptr().cast::<c_char>());
            return null_mut();
        }
    }
    let obj = (*subtype).tp_alloc.unwrap()(subtype, 0);
    Py_INCREF(data);
    (*obj.cast::<PyFragment>()).data = data;
    obj
}

#[no_mangle]
unsafe extern "C" fn fragment_dealloc(op: *mut PyObject) {
    Py_DECREF((*op.cast::<PyFragment>()).data);
    (*ob_type!(op)).tp_free.unwrap()(op.cast::<c_void>());
}

pub unsafe fn create_fragment_type() -> *mut PyTypeObject {
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
    PyType_FromSpec(&mut spec).cast::<PyTypeObject>()
}
