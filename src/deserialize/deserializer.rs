// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::DeserializeError;
use crate::exc::*;
use crate::ffi::*;
use crate::io::Read;
use crate::msgpack::Marker;
use crate::opt::*;
use crate::state::State;
use chrono::{Datelike, Timelike};
use simdutf8::basic::{from_utf8, Utf8Error};
use std::borrow::Cow;
use std::os::raw::c_char;
use std::ptr::NonNull;

const RECURSION_LIMIT: u16 = 1024;

pub fn deserialize(
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
    ext_hook: Option<NonNull<pyo3::ffi::PyObject>>,
    opts: Opt,
) -> Result<NonNull<pyo3::ffi::PyObject>, DeserializeError<'static>> {
    let obj_type_ptr = ob_type!(ptr);
    let contents: &[u8];

    if obj_type_ptr == &raw mut pyo3::ffi::PyBytes_Type {
        contents = unsafe { pybytes_as_bytes(ptr) };
    } else if obj_type_ptr == &raw mut pyo3::ffi::PyMemoryView_Type {
        if let Some(buffer) = unsafe { pymemoryview_as_bytes(ptr) } {
            contents = buffer;
        } else {
            return Err(DeserializeError::new(Cow::Borrowed(
                "Input type memoryview must be a C contiguous buffer",
            )));
        }
    } else if obj_type_ptr == &raw mut pyo3::ffi::PyByteArray_Type {
        contents = unsafe { pybytearray_as_bytes(ptr) };
    } else {
        return Err(DeserializeError::new(Cow::Borrowed(
            "Input must be bytes, bytearray, memoryview",
        )));
    }

    let mut deserializer = Deserializer::new(contents, state, ext_hook, opts);
    deserializer
        .deserialize()
        .map_err(|e| DeserializeError::new(Cow::Owned(e.to_string())))
}

#[derive(Debug)]
enum Error {
    ExtHookFailed,
    ExtHookMissing,
    Internal,
    InvalidStr,
    InvalidType(Marker),
    InvalidValue,
    RecursionLimitReached,
    UnexpectedEof,
}

impl std::fmt::Display for Error {
    #[cold]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::ExtHookFailed => f.write_str("ext_hook failed"),
            Error::ExtHookMissing => f.write_str("ext_hook missing"),
            Error::Internal => f.write_str("internal error"),
            Error::InvalidStr => f.write_str("invalid UTF-8 string"),
            Error::InvalidType(ref marker) => {
                write!(f, "invalid type {marker:?}")
            }
            Error::InvalidValue => f.write_str("invalid value"),
            Error::RecursionLimitReached => f.write_str(RECURSION_LIMIT_REACHED),
            Error::UnexpectedEof => write!(f, "unexpected end of file"),
        }
    }
}

impl From<std::io::Error> for Error {
    #[cold]
    fn from(value: std::io::Error) -> Error {
        match value.kind() {
            std::io::ErrorKind::InvalidInput => Error::InvalidValue,
            _ => Error::UnexpectedEof,
        }
    }
}

impl From<Utf8Error> for Error {
    #[cold]
    fn from(_: Utf8Error) -> Error {
        Error::InvalidStr
    }
}

struct Deserializer<R> {
    data: R,
    state: *mut State,
    ext_hook: Option<NonNull<pyo3::ffi::PyObject>>,
    opts: Opt,
    recursion: u16,
}

impl<R> Deserializer<R>
where
    R: Read,
{
    fn new(
        data: R,
        state: *mut State,
        ext_hook: Option<NonNull<pyo3::ffi::PyObject>>,
        opts: Opt,
    ) -> Self {
        Deserializer {
            data: data,
            state: state,
            ext_hook: ext_hook,
            opts: opts,
            recursion: 0,
        }
    }

    #[inline(always)]
    fn read_marker(&mut self) -> Result<Marker, Error> {
        let n = self.data.read_u8()?;
        Ok(Marker::from_u8(n))
    }

    fn deserialize_timestamp_ext(
        &mut self,
        len: u32,
    ) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        let (seconds, nanoseconds): (i64, u32) = match len {
            4 => {
                let seconds = self.data.read_u32()?;
                (seconds.into(), 0)
            }
            8 => {
                let value = self.data.read_u64()?;
                ((value & 0x3ffffffff) as i64, (value >> 34) as u32)
            }
            12 => {
                let nanoseconds = self.data.read_u32()?;
                let seconds = self.data.read_i64()?;
                (seconds, nanoseconds)
            }
            _ => return Err(Error::InvalidValue),
        };
        let datetime = match chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, nanoseconds) {
            Some(value) => value,
            None => return Err(Error::InvalidValue),
        };
        unsafe {
            let obj = {
                let datetime_api = *pyo3::ffi::PyDateTimeAPI();
                (datetime_api.DateTime_FromDateAndTime)(
                    datetime.year(),
                    datetime.month() as i32,
                    datetime.day() as i32,
                    datetime.hour() as i32,
                    datetime.minute() as i32,
                    datetime.second() as i32,
                    (datetime.nanosecond() / 1000) as i32,
                    datetime_api.TimeZone_UTC,
                    datetime_api.DateTimeType,
                )
            };
            Ok(NonNull::new_unchecked(obj))
        }
    }

    fn deserialize_ext(&mut self, len: u32) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        let tag = self.data.read_i8()?;
        if tag == -1 && self.opts & DATETIME_AS_TIMESTAMP_EXT != 0 {
            return self.deserialize_timestamp_ext(len);
        }

        let data = self.data.read_slice(len as usize)?;

        match self.ext_hook {
            Some(callable) => unsafe {
                let tag_obj = pyo3::ffi::PyLong_FromLongLong(tag as i64);
                let data_ptr = data.as_ptr().cast::<c_char>();
                let data_len = data.len() as pyo3::ffi::Py_ssize_t;
                let data_obj = pyo3::ffi::PyBytes_FromStringAndSize(data_ptr, data_len);
                let obj = pyo3::ffi::PyObject_CallFunctionObjArgs(
                    callable.as_ptr(),
                    tag_obj,
                    data_obj,
                    std::ptr::null_mut::<pyo3::ffi::PyObject>(),
                );
                pyo3::ffi::Py_DECREF(tag_obj);
                pyo3::ffi::Py_DECREF(data_obj);
                if unlikely!(obj.is_null()) {
                    Err(Error::ExtHookFailed)
                } else {
                    Ok(NonNull::new_unchecked(obj))
                }
            },
            None => Err(Error::ExtHookMissing),
        }
    }

    fn deserialize_null(&self) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        unsafe {
            let py_none = pyo3::ffi::Py_None();
            pyo3::ffi::Py_INCREF(py_none);
            Ok(NonNull::new_unchecked(py_none))
        }
    }

    fn deserialize_true(&self) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        unsafe {
            let py_true = pyo3::ffi::Py_True();
            pyo3::ffi::Py_INCREF(py_true);
            Ok(NonNull::new_unchecked(py_true))
        }
    }

    fn deserialize_false(&self) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        unsafe {
            let py_false = pyo3::ffi::Py_False();
            pyo3::ffi::Py_INCREF(py_false);
            Ok(NonNull::new_unchecked(py_false))
        }
    }

    fn deserialize_i64(&self, value: i64) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        unsafe {
            let ptr = pyo3::ffi::PyLong_FromLongLong(value);
            Ok(NonNull::new_unchecked(ptr))
        }
    }

    fn deserialize_u64(&self, value: u64) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        unsafe {
            let ptr = pyo3::ffi::PyLong_FromUnsignedLongLong(value);
            Ok(NonNull::new_unchecked(ptr))
        }
    }

    fn deserialize_f64(&self, value: f64) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        unsafe {
            let ptr = pyo3::ffi::PyFloat_FromDouble(value);
            Ok(NonNull::new_unchecked(ptr))
        }
    }

    fn deserialize_str(&mut self, len: u32) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        let data = self.data.read_slice(len as usize)?;
        let value = from_utf8(data)?;
        let ptr = unicode_from_str(value);
        unsafe { Ok(NonNull::new_unchecked(ptr)) }
    }

    fn deserialize_bin(&mut self, len: u32) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        let v = self.data.read_slice(len as usize)?;
        let ptr = v.as_ptr().cast::<c_char>();
        let len = v.len() as pyo3::ffi::Py_ssize_t;
        unsafe {
            let ptr = pyo3::ffi::PyBytes_FromStringAndSize(ptr, len);
            Ok(NonNull::new_unchecked(ptr))
        }
    }

    fn deserialize_array(&mut self, len: u32) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        let ptr = unsafe { pyo3::ffi::PyList_New(len as pyo3::ffi::Py_ssize_t) };
        for i in 0..len {
            let elem = self.deserialize()?;
            unsafe { pyo3::ffi::PyList_SET_ITEM(ptr, i as pyo3::ffi::Py_ssize_t, elem.as_ptr()) };
        }
        unsafe { Ok(NonNull::new_unchecked(ptr)) }
    }

    fn deserialize_map_with_str_keys(
        &mut self,
        len: u32,
    ) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        let dict_ptr = unsafe { pyo3::ffi::PyDict_New() };
        for _ in 0..len {
            let marker = self.read_marker()?;
            let key = match marker {
                Marker::FixStr(len) => self.deserialize_map_str_key(len.into()),
                Marker::Str8 => {
                    let len = self.data.read_u8()?;
                    self.deserialize_map_str_key(len.into())
                }
                Marker::Str16 => {
                    let len = self.data.read_u16()?;
                    self.deserialize_map_str_key(len.into())
                }
                Marker::Str32 => {
                    let len = self.data.read_u32()?;
                    self.deserialize_map_str_key(len)
                }
                marker => Err(Error::InvalidType(marker)),
            }?;
            let value = self.deserialize()?;
            unsafe {
                let _ = pyo3::ffi::PyDict_SetItem(dict_ptr, key.as_ptr(), value.as_ptr());
                // counter Py_INCREF in insertdict
                pyo3::ffi::Py_DECREF(key.as_ptr());
                pyo3::ffi::Py_DECREF(value.as_ptr());
            }
        }
        unsafe { Ok(NonNull::new_unchecked(dict_ptr)) }
    }

    fn deserialize_map_with_non_str_keys(
        &mut self,
        len: u32,
    ) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        let dict_ptr = unsafe { pyo3::ffi::PyDict_New() };
        for _ in 0..len {
            let key = self.deserialize_map_key()?;
            let value = self.deserialize()?;
            unsafe {
                let ret = pyo3::ffi::PyDict_SetItem(dict_ptr, key.as_ptr(), value.as_ptr());
                pyo3::ffi::Py_DECREF(key.as_ptr());
                pyo3::ffi::Py_DECREF(value.as_ptr());
                if unlikely!(ret == -1) {
                    return Err(Error::Internal);
                }
            }
        }
        unsafe { Ok(NonNull::new_unchecked(dict_ptr)) }
    }

    fn deserialize_map(&mut self, len: u32) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        if self.opts & NON_STR_KEYS != 0 {
            self.deserialize_map_with_non_str_keys(len)
        } else {
            self.deserialize_map_with_str_keys(len)
        }
    }

    fn deserialize(&mut self) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        self.recursion += 1;
        if unlikely!(self.recursion == RECURSION_LIMIT) {
            return Err(Error::RecursionLimitReached);
        }

        let marker = self.read_marker()?;
        let value = match marker {
            Marker::Null => self.deserialize_null(),
            Marker::True => self.deserialize_true(),
            Marker::False => self.deserialize_false(),
            Marker::FixPos(value) => self.deserialize_u64(value.into()),
            Marker::U8 => {
                let value = self.data.read_u8()?;
                self.deserialize_u64(value.into())
            }
            Marker::U16 => {
                let value = self.data.read_u16()?;
                self.deserialize_u64(value.into())
            }
            Marker::U32 => {
                let value = self.data.read_u32()?;
                self.deserialize_u64(value.into())
            }
            Marker::U64 => {
                let value = self.data.read_u64()?;
                self.deserialize_u64(value)
            }
            Marker::FixNeg(value) => self.deserialize_i64(value.into()),
            Marker::I8 => {
                let value = self.data.read_i8()?;
                self.deserialize_i64(value.into())
            }
            Marker::I16 => {
                let value = self.data.read_i16()?;
                self.deserialize_i64(value.into())
            }
            Marker::I32 => {
                let value = self.data.read_i32()?;
                self.deserialize_i64(value.into())
            }
            Marker::I64 => {
                let value = self.data.read_i64()?;
                self.deserialize_i64(value)
            }
            Marker::F32 => {
                let value = self.data.read_f32()?;
                self.deserialize_f64(value.into())
            }
            Marker::F64 => {
                let value = self.data.read_f64()?;
                self.deserialize_f64(value)
            }
            Marker::FixStr(len) => self.deserialize_str(len.into()),
            Marker::Str8 => {
                let len = self.data.read_u8()?;
                self.deserialize_str(len.into())
            }
            Marker::Str16 => {
                let len = self.data.read_u16()?;
                self.deserialize_str(len.into())
            }
            Marker::Str32 => {
                let len = self.data.read_u32()?;
                self.deserialize_str(len)
            }
            Marker::Bin8 => {
                let len = self.data.read_u8()?;
                self.deserialize_bin(len.into())
            }
            Marker::Bin16 => {
                let len = self.data.read_u16()?;
                self.deserialize_bin(len.into())
            }
            Marker::Bin32 => {
                let len = self.data.read_u32()?;
                self.deserialize_bin(len)
            }
            Marker::FixArray(len) => self.deserialize_array(len.into()),
            Marker::Array16 => {
                let len = self.data.read_u16()?;
                self.deserialize_array(len.into())
            }
            Marker::Array32 => {
                let len = self.data.read_u32()?;
                self.deserialize_array(len)
            }
            Marker::FixMap(len) => self.deserialize_map(len.into()),
            Marker::Map16 => {
                let len = self.data.read_u16()?;
                self.deserialize_map(len.into())
            }
            Marker::Map32 => {
                let len = self.data.read_u32()?;
                self.deserialize_map(len)
            }
            Marker::FixExt1 => self.deserialize_ext(1),
            Marker::FixExt2 => self.deserialize_ext(2),
            Marker::FixExt4 => self.deserialize_ext(4),
            Marker::FixExt8 => self.deserialize_ext(8),
            Marker::FixExt16 => self.deserialize_ext(16),
            Marker::Ext8 => {
                let len = self.data.read_u8()?;
                self.deserialize_ext(len.into())
            }
            Marker::Ext16 => {
                let len = self.data.read_u16()?;
                self.deserialize_ext(len.into())
            }
            Marker::Ext32 => {
                let len = self.data.read_u32()?;
                self.deserialize_ext(len)
            }
            Marker::Reserved => Err(Error::InvalidType(Marker::Reserved)),
        };

        self.recursion -= 1;
        value
    }

    fn deserialize_map_str_key(&mut self, len: u32) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        if unlikely!(len > 64) {
            let value = self.deserialize_str(len)?;
            hash_str(value.as_ptr());
            Ok(value)
        } else {
            let data = self.data.read_slice(len as usize)?;
            Ok(unsafe { (*self.state).key_map.get(data)? })
        }
    }

    fn deserialize_map_array_key(
        &mut self,
        len: u32,
    ) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        let ptr = unsafe { pyo3::ffi::PyTuple_New(len as pyo3::ffi::Py_ssize_t) };
        for i in 0..len {
            let elem = self.deserialize_map_key()?;
            unsafe {
                pytuple_set_item(ptr, i as pyo3::ffi::Py_ssize_t, elem.as_ptr());
            }
        }
        unsafe { Ok(NonNull::new_unchecked(ptr)) }
    }

    fn deserialize_map_ext_key(&mut self, len: u32) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        let tag = self.data.read_i8()?;
        if tag == -1 && self.opts & DATETIME_AS_TIMESTAMP_EXT != 0 {
            self.deserialize_timestamp_ext(len)
        } else {
            Err(Error::InvalidValue)
        }
    }

    fn deserialize_map_key(&mut self) -> Result<NonNull<pyo3::ffi::PyObject>, Error> {
        self.recursion += 1;
        if unlikely!(self.recursion == RECURSION_LIMIT) {
            return Err(Error::RecursionLimitReached);
        }

        let marker = self.read_marker()?;
        let value = match marker {
            Marker::Null => self.deserialize_null(),
            Marker::True => self.deserialize_true(),
            Marker::False => self.deserialize_false(),
            Marker::FixPos(value) => self.deserialize_u64(value.into()),
            Marker::U8 => {
                let value = self.data.read_u8()?;
                self.deserialize_u64(value.into())
            }
            Marker::U16 => {
                let value = self.data.read_u16()?;
                self.deserialize_u64(value.into())
            }
            Marker::U32 => {
                let value = self.data.read_u32()?;
                self.deserialize_u64(value.into())
            }
            Marker::U64 => {
                let value = self.data.read_u64()?;
                self.deserialize_u64(value)
            }
            Marker::FixNeg(value) => self.deserialize_i64(value.into()),
            Marker::I8 => {
                let value = self.data.read_i8()?;
                self.deserialize_i64(value.into())
            }
            Marker::I16 => {
                let value = self.data.read_i16()?;
                self.deserialize_i64(value.into())
            }
            Marker::I32 => {
                let value = self.data.read_i32()?;
                self.deserialize_i64(value.into())
            }
            Marker::I64 => {
                let value = self.data.read_i64()?;
                self.deserialize_i64(value)
            }
            Marker::F32 => {
                let value = self.data.read_f32()?;
                self.deserialize_f64(value.into())
            }
            Marker::F64 => {
                let value = self.data.read_f64()?;
                self.deserialize_f64(value)
            }
            Marker::FixStr(len) => self.deserialize_map_str_key(len.into()),
            Marker::Str8 => {
                let len = self.data.read_u8()?;
                self.deserialize_map_str_key(len.into())
            }
            Marker::Str16 => {
                let len = self.data.read_u16()?;
                self.deserialize_map_str_key(len.into())
            }
            Marker::Str32 => {
                let len = self.data.read_u32()?;
                self.deserialize_map_str_key(len)
            }
            Marker::Bin8 => {
                let len = self.data.read_u8()?;
                self.deserialize_bin(len.into())
            }
            Marker::Bin16 => {
                let len = self.data.read_u16()?;
                self.deserialize_bin(len.into())
            }
            Marker::Bin32 => {
                let len = self.data.read_u32()?;
                self.deserialize_bin(len)
            }
            Marker::FixArray(len) => self.deserialize_map_array_key(len.into()),
            Marker::Array16 => {
                let len = self.data.read_u16()?;
                self.deserialize_map_array_key(len.into())
            }
            Marker::Array32 => {
                let len = self.data.read_u32()?;
                self.deserialize_map_array_key(len)
            }
            Marker::FixExt4 => self.deserialize_map_ext_key(4),
            Marker::FixExt8 => self.deserialize_map_ext_key(8),
            Marker::Ext8 => {
                let len = self.data.read_u8()?;
                self.deserialize_map_ext_key(len.into())
            }
            marker => Err(Error::InvalidType(marker)),
        };

        self.recursion -= 1;
        value
    }
}
