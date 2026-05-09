// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[cfg(unicode_state)]
use crate::ffi::impl_::unicode_state::*;
use crate::ffi::unicode::*;
use crate::str::count_chars;
use pyo3::ffi::*;

// see unicodeobject.h for documentation

pub fn unicode_from_str(buf: &str) -> *mut PyObject {
    if buf.is_empty() {
        unsafe { PyUnicode_New(0, 0) }
    } else {
        let num_chars = count_chars(buf.as_bytes());
        if buf.len() == num_chars {
            pyunicode_ascii(buf)
        } else {
            let max = buf.bytes().max().unwrap();
            if max >= 0xf0 {
                pyunicode_fourbyte(buf, num_chars)
            } else if max >= 0xc4 {
                pyunicode_twobyte(buf, num_chars)
            } else {
                pyunicode_onebyte(buf, num_chars)
            }
        }
    }
}

fn pyunicode_ascii(buf: &str) -> *mut PyObject {
    unsafe {
        let ptr = PyUnicode_New(buf.len() as isize, 127);
        let data_ptr = ptr.cast::<PyASCIIObject>().offset(1).cast::<u8>();
        std::ptr::copy_nonoverlapping(buf.as_ptr(), data_ptr, buf.len());
        std::ptr::write(data_ptr.add(buf.len()), 0);
        ptr
    }
}

#[cold]
#[inline(never)]
fn pyunicode_onebyte(buf: &str, num_chars: usize) -> *mut PyObject {
    unsafe {
        let ptr = PyUnicode_New(num_chars as isize, 255);
        let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1).cast::<u8>();
        for each in buf.chars() {
            std::ptr::write(data_ptr, each as u8);
            data_ptr = data_ptr.offset(1);
        }
        std::ptr::write(data_ptr, 0);
        ptr
    }
}

fn pyunicode_twobyte(buf: &str, num_chars: usize) -> *mut PyObject {
    unsafe {
        let ptr = PyUnicode_New(num_chars as isize, 65535);
        let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1).cast::<u16>();
        for each in buf.chars() {
            std::ptr::write(data_ptr, each as u16);
            data_ptr = data_ptr.offset(1);
        }
        std::ptr::write(data_ptr, 0);
        ptr
    }
}

fn pyunicode_fourbyte(buf: &str, num_chars: usize) -> *mut PyObject {
    unsafe {
        let ptr = PyUnicode_New(num_chars as isize, 1114111);
        let mut data_ptr = ptr.cast::<PyCompactUnicodeObject>().offset(1).cast::<u32>();
        for each in buf.chars() {
            std::ptr::write(data_ptr, each as u32);
            data_ptr = data_ptr.offset(1);
        }
        std::ptr::write(data_ptr, 0);
        ptr
    }
}

#[cfg(unicode_state)]
#[inline]
pub fn hash_str(op: *mut PyObject) -> Py_hash_t {
    unsafe {
        let ptr = pyunicode_compact_data(op);
        let len = (*op.cast::<PyASCIIObject>()).length * pyunicode_kind(op) as Py_ssize_t;
        let hash = compat::Py_HashBuffer(ptr, len);
        (*op.cast::<PyASCIIObject>()).hash = hash;
        hash
    }
}

#[cfg(not(unicode_state))]
#[inline]
pub fn hash_str(op: *mut PyObject) -> Py_hash_t {
    unsafe {
        let ptr = PyUnicode_DATA(op);
        let len = (*op.cast::<PyASCIIObject>()).length * PyUnicode_KIND(op) as Py_ssize_t;
        let hash = compat::Py_HashBuffer(ptr, len);
        (*op.cast::<PyASCIIObject>()).hash = hash;
        hash
    }
}

#[cfg(unicode_state)]
#[inline]
pub fn unicode_to_str(op: *mut PyObject) -> Result<&'static str, UnicodeError> {
    unsafe {
        if unlikely!(!pyunicode_is_compact(op)) {
            unicode_to_str_via_ffi(op)
        } else if pyunicode_is_ascii(op) {
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
            unicode_to_str_via_ffi(op)
        }
    }
}

#[cfg(not(unicode_state))]
#[inline]
pub fn unicode_to_str(op: *mut PyObject) -> Result<&'static str, UnicodeError> {
    unicode_to_str_via_ffi(op)
}
