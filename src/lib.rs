// SPDX-License-Identifier: (Apache-2.0 OR MIT)
#![cfg_attr(feature = "unstable-simd", feature(core_intrinsics))]
#![allow(internal_features)]
#![allow(unused_unsafe)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::ptr_eq)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::zero_prefixed_literal)]
#![deny(clippy::ptr_as_ptr)]

#[macro_use]
mod util;

mod deserialize;
mod exc;
mod ext;
mod ffi;
mod fragment;
mod io;
mod msgpack;
mod opt;
mod serialize;
mod state;

use crate::ffi::*;
use pyo3::ffi::*;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_long;
use std::os::raw::c_void;
use std::ptr::NonNull;

const PACKB_DOC: &CStr =
    c"packb(obj, /, default=None, option=None)\n--\n\nSerialize Python objects to msgpack.";
const UNPACKB_DOC: &CStr =
    c"unpackb(obj, /, *, ext_hook=None, option=None)\n--\n\nDeserialize msgpack to Python objects.";

macro_rules! module_add_object {
    ($mptr: expr, $name: expr, $object:expr) => {
        PyModule_AddObject($mptr, $name.as_ptr(), $object);
    };
}

macro_rules! module_add_int {
    ($mptr:expr, $name:expr, $int:expr) => {
        PyModule_AddIntConstant($mptr, $name.as_ptr(), $int as c_long);
    };
}

#[allow(non_snake_case)]
#[no_mangle]
#[cold]
pub unsafe extern "C" fn PyInit_ormsgpack() -> *mut PyModuleDef {
    let methods: Box<[PyMethodDef; 3]> = Box::new([
        PyMethodDef {
            ml_name: c"packb".as_ptr(),
            ml_meth: PyMethodDefPointer {
                PyCFunctionFastWithKeywords: packb,
            },
            ml_flags: METH_FASTCALL | METH_KEYWORDS,
            ml_doc: PACKB_DOC.as_ptr(),
        },
        PyMethodDef {
            ml_name: c"unpackb".as_ptr(),
            ml_meth: PyMethodDefPointer {
                PyCFunctionFastWithKeywords: unpackb,
            },
            ml_flags: METH_FASTCALL | METH_KEYWORDS,
            ml_doc: UNPACKB_DOC.as_ptr(),
        },
        PyMethodDef::zeroed(),
    ]);

    let slots: Box<[PyModuleDef_Slot]> = Box::new([
        PyModuleDef_Slot {
            slot: Py_mod_exec,
            value: ormsgpack_exec as *mut c_void,
        },
        #[cfg(Py_3_12)]
        PyModuleDef_Slot {
            slot: Py_mod_multiple_interpreters,
            value: Py_MOD_PER_INTERPRETER_GIL_SUPPORTED,
        },
        #[cfg(Py_3_13)]
        PyModuleDef_Slot {
            slot: Py_mod_gil,
            value: Py_MOD_GIL_NOT_USED,
        },
        PyModuleDef_Slot {
            slot: 0,
            value: std::ptr::null_mut(),
        },
    ]);

    let init = Box::new(PyModuleDef {
        m_base: PyModuleDef_HEAD_INIT,
        m_name: c"ormsgpack".as_ptr(),
        m_doc: std::ptr::null(),
        m_size: std::mem::size_of::<state::State>() as Py_ssize_t,
        m_methods: Box::into_raw(methods).cast::<PyMethodDef>(),
        m_slots: Box::into_raw(slots).cast::<PyModuleDef_Slot>(),
        m_traverse: None,
        m_clear: None,
        m_free: None,
    });
    let init_ptr = Box::into_raw(init);
    PyModuleDef_Init(init_ptr);
    init_ptr
}

#[allow(non_snake_case)]
#[no_mangle]
#[cold]
pub unsafe extern "C" fn ormsgpack_exec(mptr: *mut PyObject) -> c_int {
    PyDateTime_IMPORT();

    let state: *mut state::State = PyModule_GetState(mptr).cast();
    *state = state::State::new();

    let version = env!("CARGO_PKG_VERSION");
    module_add_object!(
        mptr,
        c"__version__",
        PyUnicode_FromStringAndSize(version.as_ptr().cast::<c_char>(), version.len() as isize)
    );
    module_add_object!(mptr, c"Ext", (*state).ext_type.cast::<PyObject>());
    module_add_object!(mptr, c"Fragment", (*state).fragment_type.cast::<PyObject>());
    module_add_object!(mptr, c"MsgpackDecodeError", (*state).MsgpackDecodeError);
    module_add_object!(mptr, c"MsgpackEncodeError", (*state).MsgpackEncodeError);

    module_add_int!(
        mptr,
        c"OPT_DATETIME_AS_TIMESTAMP_EXT",
        opt::DATETIME_AS_TIMESTAMP_EXT
    );
    module_add_int!(mptr, c"OPT_NAIVE_UTC", opt::NAIVE_UTC);
    module_add_int!(mptr, c"OPT_NON_STR_KEYS", opt::NON_STR_KEYS);
    module_add_int!(mptr, c"OPT_OMIT_MICROSECONDS", opt::OMIT_MICROSECONDS);
    module_add_int!(mptr, c"OPT_PASSTHROUGH_BIG_INT", opt::PASSTHROUGH_BIG_INT);
    module_add_int!(
        mptr,
        c"OPT_PASSTHROUGH_DATACLASS",
        opt::PASSTHROUGH_DATACLASS
    );
    module_add_int!(mptr, c"OPT_PASSTHROUGH_DATETIME", opt::PASSTHROUGH_DATETIME);
    module_add_int!(mptr, c"OPT_PASSTHROUGH_ENUM", opt::PASSTHROUGH_ENUM);
    module_add_int!(mptr, c"OPT_PASSTHROUGH_SUBCLASS", opt::PASSTHROUGH_SUBCLASS);
    module_add_int!(mptr, c"OPT_PASSTHROUGH_TUPLE", opt::PASSTHROUGH_TUPLE);
    module_add_int!(mptr, c"OPT_PASSTHROUGH_UUID", opt::PASSTHROUGH_UUID);
    module_add_int!(mptr, c"OPT_REPLACE_SURROGATES", opt::REPLACE_SURROGATES);
    module_add_int!(mptr, c"OPT_SERIALIZE_NUMPY", opt::SERIALIZE_NUMPY);
    module_add_int!(mptr, c"OPT_SERIALIZE_PYDANTIC", opt::SERIALIZE_PYDANTIC);
    module_add_int!(mptr, c"OPT_SORT_KEYS", opt::SORT_KEYS);
    module_add_int!(mptr, c"OPT_UTC_Z", opt::UTC_Z);

    0
}

#[cold]
#[inline(never)]
fn raise_unpackb_exception(state: *mut state::State, msg: &str) -> *mut PyObject {
    unsafe {
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr().cast::<c_char>(), msg.len() as isize);
        let args = PyTuple_New(1);
        pytuple_set_item(args, 0, err_msg);
        PyErr_SetObject((*state).MsgpackDecodeError, args);
        Py_DECREF(args);
    };
    std::ptr::null_mut()
}

#[cold]
#[inline(never)]
fn raise_packb_exception(state: *mut state::State, msg: &str) -> *mut PyObject {
    unsafe {
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr().cast::<c_char>(), msg.len() as isize);
        PyErr_SetObject((*state).MsgpackEncodeError, err_msg);
        Py_DECREF(err_msg);
    };
    std::ptr::null_mut()
}

unsafe fn parse_option_arg(opts: *mut PyObject, mask: i32) -> Result<i32, ()> {
    if Py_TYPE(opts) == &raw mut PyLong_Type {
        let val = PyLong_AsLong(opts) as i32;
        if val & !mask == 0 {
            Ok(val)
        } else {
            Err(())
        }
    } else if opts == Py_None() {
        Ok(0)
    } else {
        Err(())
    }
}

#[no_mangle]
pub unsafe extern "C" fn unpackb(
    module: *mut PyObject,
    args: *const *mut PyObject,
    nargs: Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject {
    let state: *mut state::State = PyModule_GetState(module).cast();
    let mut ext_hook: Option<NonNull<PyObject>> = None;
    let mut optsptr: Option<NonNull<PyObject>> = None;

    let num_args = PyVectorcall_NARGS(nargs as usize);
    if unlikely!(num_args != 1) {
        let msg = if num_args > 1 {
            "unpackb() accepts only 1 positional argument"
        } else {
            "unpackb() missing 1 required positional argument: 'obj'"
        };
        return raise_unpackb_exception(state, msg);
    }
    if !kwnames.is_null() {
        let tuple_size = Py_SIZE(kwnames);
        for i in 0..tuple_size {
            let arg = pytuple_get_item(kwnames, i as Py_ssize_t);
            if PyUnicode_Compare(arg, (*state).ext_hook_str) == 0 {
                ext_hook = Some(NonNull::new_unchecked(*args.offset(num_args + i)));
            } else if PyUnicode_Compare(arg, (*state).option_str) == 0 {
                optsptr = Some(NonNull::new_unchecked(*args.offset(num_args + i)));
            } else {
                return raise_unpackb_exception(
                    state,
                    "unpackb() got an unexpected keyword argument",
                );
            }
        }
    }

    let mut optsbits: i32 = 0;
    if let Some(opts) = optsptr {
        match parse_option_arg(opts.as_ptr(), opt::UNPACKB_OPT_MASK) {
            Ok(val) => optsbits = val,
            Err(()) => return raise_unpackb_exception(state, "Invalid opts"),
        }
    }

    match crate::deserialize::deserialize(*args, state, ext_hook, optsbits as opt::Opt) {
        Ok(val) => val.as_ptr(),
        Err(err) => raise_unpackb_exception(state, &err.message),
    }
}

#[no_mangle]
pub unsafe extern "C" fn packb(
    module: *mut PyObject,
    args: *const *mut PyObject,
    nargs: Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject {
    let state: *mut state::State = PyModule_GetState(module).cast();
    let mut default: Option<NonNull<PyObject>> = None;
    let mut optsptr: Option<NonNull<PyObject>> = None;

    let num_args = PyVectorcall_NARGS(nargs as usize);
    if unlikely!(num_args == 0) {
        return raise_packb_exception(
            state,
            "packb() missing 1 required positional argument: 'obj'",
        );
    }
    if num_args >= 2 {
        default = Some(NonNull::new_unchecked(*args.offset(1)));
    }
    if num_args >= 3 {
        optsptr = Some(NonNull::new_unchecked(*args.offset(2)));
    }
    if !kwnames.is_null() {
        let tuple_size = Py_SIZE(kwnames);
        for i in 0..tuple_size {
            let arg = pytuple_get_item(kwnames, i as Py_ssize_t);
            if PyUnicode_Compare(arg, (*state).default_str) == 0 {
                if unlikely!(default.is_some()) {
                    return raise_packb_exception(
                        state,
                        "packb() got multiple values for argument: 'default'",
                    );
                }
                default = Some(NonNull::new_unchecked(*args.offset(num_args + i)));
            } else if PyUnicode_Compare(arg, (*state).option_str) == 0 {
                if unlikely!(optsptr.is_some()) {
                    return raise_packb_exception(
                        state,
                        "packb() got multiple values for argument: 'option'",
                    );
                }
                optsptr = Some(NonNull::new_unchecked(*args.offset(num_args + i)));
            } else {
                return raise_packb_exception(state, "packb() got an unexpected keyword argument");
            }
        }
    }

    let mut optsbits: i32 = 0;
    if let Some(opts) = optsptr {
        match parse_option_arg(opts.as_ptr(), opt::PACKB_OPT_MASK) {
            Ok(val) => optsbits = val,
            Err(()) => return raise_packb_exception(state, "Invalid opts"),
        }
    }

    match crate::serialize::serialize(*args, state, default, optsbits as opt::Opt) {
        Ok(val) => val.as_ptr(),
        Err(err) => raise_packb_exception(state, &err),
    }
}
