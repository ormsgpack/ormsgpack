// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[cfg(unicode_state)]
use crate::ffi::impl_::unicode_state::*;
use crate::ffi::unicode::*;
use crate::str::count_chars;
use crate::util::unlikely;
use pyo3::ffi::*;
use pyo3::prelude::*;
use pyo3::types::PyString;

// see unicodeobject.h for documentation

pub fn unicode_from_str<'py>(py: Python<'py>, buf: &str) -> Bound<'py, PyString> {
    if buf.is_empty() {
        unsafe { Bound::from_owned_ptr(py, PyUnicode_New(0, 0)).cast_into_unchecked() }
    } else {
        let num_chars = count_chars(buf.as_bytes());
        if buf.len() == num_chars {
            pyunicode_ascii(py, buf)
        } else {
            let max = buf.bytes().max().unwrap();
            if max >= 0xf0 {
                pyunicode_fourbyte(py, buf, num_chars)
            } else if max >= 0xc4 {
                pyunicode_twobyte(py, buf, num_chars)
            } else {
                pyunicode_onebyte(py, buf, num_chars)
            }
        }
    }
}

fn pyunicode_ascii<'py>(py: Python<'py>, buf: &str) -> Bound<'py, PyString> {
    let obj = unsafe {
        Bound::from_owned_ptr(py, PyUnicode_New(buf.len() as isize, 127)).cast_into_unchecked()
    };
    unsafe {
        let ptr = obj.as_ptr();
        let data_ptr = ptr.cast::<PyASCIIObject>().offset(1).cast::<u8>();
        std::ptr::copy_nonoverlapping(buf.as_ptr(), data_ptr, buf.len());
        std::ptr::write(data_ptr.add(buf.len()), 0);
    }
    obj
}

#[cold]
#[inline(never)]
fn pyunicode_onebyte<'py>(py: Python<'py>, buf: &str, num_chars: usize) -> Bound<'py, PyString> {
    let obj = unsafe {
        Bound::from_owned_ptr(py, PyUnicode_New(num_chars as isize, 255)).cast_into_unchecked()
    };
    unsafe {
        let ptr = obj.as_ptr();
        let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1).cast::<u8>();
        for each in buf.chars() {
            std::ptr::write(data_ptr, each as u8);
            data_ptr = data_ptr.offset(1);
        }
        std::ptr::write(data_ptr, 0);
    }
    obj
}

fn pyunicode_twobyte<'py>(py: Python<'py>, buf: &str, num_chars: usize) -> Bound<'py, PyString> {
    let obj = unsafe {
        Bound::from_owned_ptr(py, PyUnicode_New(num_chars as isize, 65535)).cast_into_unchecked()
    };
    unsafe {
        let ptr = obj.as_ptr();
        let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1).cast::<u16>();
        for each in buf.chars() {
            std::ptr::write(data_ptr, each as u16);
            data_ptr = data_ptr.offset(1);
        }
        std::ptr::write(data_ptr, 0);
    }
    obj
}

fn pyunicode_fourbyte<'py>(py: Python<'py>, buf: &str, num_chars: usize) -> Bound<'py, PyString> {
    let obj = unsafe {
        Bound::from_owned_ptr(py, PyUnicode_New(num_chars as isize, 1114111)).cast_into_unchecked()
    };
    unsafe {
        let ptr = obj.as_ptr();
        let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1).cast::<u32>();
        for each in buf.chars() {
            std::ptr::write(data_ptr, each as u32);
            data_ptr = data_ptr.offset(1);
        }
        std::ptr::write(data_ptr, 0);
    }
    obj
}

#[cfg(unicode_state)]
#[inline]
pub fn hash_str(obj: Borrowed<'_, '_, PyString>) -> Py_hash_t {
    unsafe {
        let op = obj.as_ptr();
        let ptr = pyunicode_compact_data(obj);
        let len = (*op.cast::<PyASCIIObject>()).length * pyunicode_kind(obj) as Py_ssize_t;
        let hash = compat::Py_HashBuffer(ptr, len);
        (*op.cast::<PyASCIIObject>()).hash = hash;
        hash
    }
}

#[cfg(not(unicode_state))]
#[inline]
pub fn hash_str(obj: Borrowed<'_, '_, PyString>) -> Py_hash_t {
    unsafe {
        let op = obj.as_ptr();
        let ptr = PyUnicode_DATA(op);
        let len = (*op.cast::<PyASCIIObject>()).length * PyUnicode_KIND(op) as Py_ssize_t;
        let hash = compat::Py_HashBuffer(ptr, len);
        (*op.cast::<PyASCIIObject>()).hash = hash;
        hash
    }
}

#[cfg(unicode_state)]
#[inline]
pub fn unicode_to_str<'a>(obj: Borrowed<'a, '_, PyString>) -> Result<&'a str, UnicodeError> {
    unsafe {
        let op = obj.as_ptr();
        if unlikely(!pyunicode_is_compact(obj)) {
            unicode_to_str_via_ffi(obj)
        } else if pyunicode_is_ascii(obj) {
            let ptr = op.cast::<PyASCIIObject>().offset(1).cast::<u8>();
            let len = (*op.cast::<PyASCIIObject>()).length as usize;
            let slice = std::slice::from_raw_parts(ptr, len);
            Ok(std::str::from_utf8_unchecked(slice))
        } else if (*op.cast::<PyCompactUnicodeObject>()).utf8_length != 0 {
            let ptr = (*op.cast::<PyCompactUnicodeObject>()).utf8.cast::<u8>();
            let len = (*op.cast::<PyCompactUnicodeObject>()).utf8_length as usize;
            let slice = std::slice::from_raw_parts(ptr, len);
            Ok(std::str::from_utf8_unchecked(slice))
        } else {
            unicode_to_str_via_ffi(obj)
        }
    }
}

#[cfg(not(unicode_state))]
#[inline]
pub fn unicode_to_str<'a>(obj: Borrowed<'a, '_, PyString>) -> Result<&'a str, UnicodeError> {
    unicode_to_str_via_ffi(obj)
}
