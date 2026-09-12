// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::state::State;
use crate::exc::*;
use crate::ffi::*;
use crate::io::Read;
use crate::msgpack::{read_timestamp, Marker};
use crate::opt::*;
use crate::util::unlikely;
use chrono::{Datelike, Timelike};
use pyo3::prelude::*;
use pyo3::types::{
    PyBool, PyByteArray, PyBytes, PyDateTime, PyDict, PyFloat, PyInt, PyList, PyMemoryView, PyNone,
    PyString, PyTuple, PyTzInfo,
};
use simdutf8::basic::{from_utf8, Utf8Error};

struct PyItem<'py>(PyResult<Bound<'py, PyAny>>);

impl<'py> IntoPyObject<'py> for PyItem<'py> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, _py: Python<'py>) -> PyResult<Self::Output> {
        self.0
    }
}

const RECURSION_LIMIT: u16 = 1024;

fn deserialize_slice<'a, 'py>(
    py: Python<'py>,
    contents: &[u8],
    state: &State,
    ext_hook: Option<Borrowed<'a, 'py, PyAny>>,
    opts: Opt,
) -> PyResult<Bound<'py, PyAny>> {
    let mut deserializer = Deserializer::new(py, contents, state, ext_hook, opts);
    deserializer
        .deserialize()
        .map_err(|e| e.into_pyerr(py, state))
}

pub fn deserialize<'a, 'py>(
    obj: Borrowed<'a, 'py, PyAny>,
    state: &State,
    ext_hook: Option<Borrowed<'a, 'py, PyAny>>,
    opts: Opt,
) -> PyResult<Bound<'py, PyAny>> {
    let py = obj.py();
    let obj = BorrowedWithType::new(obj);

    if let Some(obj) = obj.cast_exact::<PyBytes>() {
        let contents = obj.as_bytes();
        deserialize_slice(py, contents, state, ext_hook, opts)
    } else if let Some(obj) = obj.cast_exact::<PyMemoryView>() {
        if let Some(buffer) = Buffer::get(obj) {
            let contents = buffer.as_bytes();
            deserialize_slice(py, contents, state, ext_hook, opts)
        } else {
            Err(state.error(py, "Input type memoryview must be a C contiguous buffer"))
        }
    } else if let Some(obj) = obj.cast_exact::<PyByteArray>() {
        let contents = unsafe { obj.as_bytes() };
        deserialize_slice(py, contents, state, ext_hook, opts)
    } else {
        Err(state.error(py, "Input must be bytes, bytearray, memoryview"))
    }
}

#[derive(Debug)]
enum Error {
    ExtHookFailed,
    ExtHookMissing,
    Internal,
    InvalidStr,
    InvalidType(Marker),
    InvalidValue,
    PyErr(PyErr),
    RecursionLimitReached,
    UnexpectedEof,
}

impl Error {
    #[cold]
    fn into_pyerr(self, py: Python<'_>, state: &State) -> PyErr {
        match self {
            Self::PyErr(err) => err,
            Self::ExtHookFailed => state.error(py, "ext_hook failed"),
            Self::ExtHookMissing => state.error(py, "ext_hook missing"),
            Self::Internal => state.error(py, "internal error"),
            Self::InvalidStr => state.error(py, "invalid UTF-8 string"),
            Self::InvalidType(marker) => state.error(py, &format!("invalid type {marker:?}")),
            Self::InvalidValue => state.error(py, "invalid value"),
            Self::RecursionLimitReached => state.error(py, RECURSION_LIMIT_REACHED),
            Self::UnexpectedEof => state.error(py, "unexpected end of file"),
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

struct Deserializer<'a, 'py, R> {
    py: Python<'py>,
    data: R,
    state: &'a State,
    ext_hook: Option<Borrowed<'a, 'py, PyAny>>,
    opts: Opt,
    recursion: u16,
}

impl<'a, 'py, R> Deserializer<'a, 'py, R>
where
    R: Read,
{
    fn new(
        py: Python<'py>,
        data: R,
        state: &'a State,
        ext_hook: Option<Borrowed<'a, 'py, PyAny>>,
        opts: Opt,
    ) -> Self {
        Deserializer {
            py: py,
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

    fn deserialize_timestamp_ext(&mut self, len: u32) -> Result<Bound<'py, PyDateTime>, Error> {
        let datetime = read_timestamp(&mut self.data, len)?;
        let utc = PyTzInfo::utc(self.py).map_err(|_| Error::Internal)?;
        PyDateTime::new(
            self.py,
            datetime.year(),
            datetime.month() as u8,
            datetime.day() as u8,
            datetime.hour() as u8,
            datetime.minute() as u8,
            datetime.second() as u8,
            datetime.nanosecond() / 1000,
            Some(&utc),
        )
        .map_err(|_| Error::Internal)
    }

    fn deserialize_ext(&mut self, len: u32) -> Result<Bound<'py, PyAny>, Error> {
        let tag = self.data.read_i8()?;
        if tag == -1 && self.opts & DATETIME_AS_TIMESTAMP_EXT != 0 {
            return self.deserialize_timestamp_ext(len).map(Bound::into_any);
        }

        let data = self.data.read_slice(len as usize)?;

        match self.ext_hook {
            Some(callable) => {
                let tag_obj = tag.into_pyobject(self.py).unwrap();
                let data_obj = PyBytes::new(self.py, data);
                callable
                    .call((tag_obj, data_obj), None)
                    .map_err(|_| Error::ExtHookFailed)
            }
            None => Err(Error::ExtHookMissing),
        }
    }

    fn deserialize_null(&self) -> Result<Bound<'py, PyNone>, Error> {
        Ok(PyNone::get(self.py).to_owned())
    }

    fn deserialize_true(&self) -> Result<Bound<'py, PyBool>, Error> {
        Ok(PyBool::new(self.py, true).to_owned())
    }

    fn deserialize_false(&self) -> Result<Bound<'py, PyBool>, Error> {
        Ok(PyBool::new(self.py, false).to_owned())
    }

    fn deserialize_i64(&self, value: i64) -> Result<Bound<'py, PyInt>, Error> {
        Ok(value.into_pyobject(self.py).unwrap())
    }

    fn deserialize_u64(&self, value: u64) -> Result<Bound<'py, PyInt>, Error> {
        Ok(value.into_pyobject(self.py).unwrap())
    }

    fn deserialize_f64(&self, value: f64) -> Result<Bound<'py, PyFloat>, Error> {
        Ok(value.into_pyobject(self.py).unwrap())
    }

    fn deserialize_str(&mut self, len: u32) -> Result<Bound<'py, PyString>, Error> {
        let data = self.data.read_slice(len as usize)?;
        let value = from_utf8(data)?;
        Ok(unicode_from_str(self.py, value))
    }

    fn deserialize_bin(&mut self, len: u32) -> Result<Bound<'py, PyBytes>, Error> {
        let v = self.data.read_slice(len as usize)?;
        Ok(PyBytes::new(self.py, v))
    }

    fn deserialize_array(&mut self, len: u32) -> Result<Bound<'py, PyList>, Error> {
        let py = self.py;
        let state = self.state;
        let elements =
            (0..len).map(|_| PyItem(self.deserialize().map_err(|e| e.into_pyerr(py, state))));
        PyList::new(py, elements).map_err(Error::PyErr)
    }

    fn deserialize_map_with_str_keys(&mut self, len: u32) -> Result<Bound<'py, PyDict>, Error> {
        let dict = PyDict::new(self.py);
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
            dict.set_item(key, value).map_err(|_| Error::Internal)?;
        }
        Ok(dict)
    }

    fn deserialize_map_with_non_str_keys(&mut self, len: u32) -> Result<Bound<'py, PyDict>, Error> {
        let dict = PyDict::new(self.py);
        for _ in 0..len {
            let key = self.deserialize_map_key()?;
            let value = self.deserialize()?;
            dict.set_item(key, value).map_err(|_| Error::Internal)?;
        }
        Ok(dict)
    }

    fn deserialize_map(&mut self, len: u32) -> Result<Bound<'py, PyDict>, Error> {
        if self.opts & NON_STR_KEYS != 0 {
            self.deserialize_map_with_non_str_keys(len)
        } else {
            self.deserialize_map_with_str_keys(len)
        }
    }

    fn deserialize(&mut self) -> Result<Bound<'py, PyAny>, Error> {
        self.recursion += 1;
        if unlikely(self.recursion == RECURSION_LIMIT) {
            return Err(Error::RecursionLimitReached);
        }

        let marker = self.read_marker()?;
        let value = match marker {
            Marker::Null => self.deserialize_null().map(Bound::into_any),
            Marker::True => self.deserialize_true().map(Bound::into_any),
            Marker::False => self.deserialize_false().map(Bound::into_any),
            Marker::FixPos(value) => self.deserialize_u64(value.into()).map(Bound::into_any),
            Marker::U8 => {
                let value = self.data.read_u8()?;
                self.deserialize_u64(value.into()).map(Bound::into_any)
            }
            Marker::U16 => {
                let value = self.data.read_u16()?;
                self.deserialize_u64(value.into()).map(Bound::into_any)
            }
            Marker::U32 => {
                let value = self.data.read_u32()?;
                self.deserialize_u64(value.into()).map(Bound::into_any)
            }
            Marker::U64 => {
                let value = self.data.read_u64()?;
                self.deserialize_u64(value).map(Bound::into_any)
            }
            Marker::FixNeg(value) => self.deserialize_i64(value.into()).map(Bound::into_any),
            Marker::I8 => {
                let value = self.data.read_i8()?;
                self.deserialize_i64(value.into()).map(Bound::into_any)
            }
            Marker::I16 => {
                let value = self.data.read_i16()?;
                self.deserialize_i64(value.into()).map(Bound::into_any)
            }
            Marker::I32 => {
                let value = self.data.read_i32()?;
                self.deserialize_i64(value.into()).map(Bound::into_any)
            }
            Marker::I64 => {
                let value = self.data.read_i64()?;
                self.deserialize_i64(value).map(Bound::into_any)
            }
            Marker::F32 => {
                let value = self.data.read_f32()?;
                self.deserialize_f64(value.into()).map(Bound::into_any)
            }
            Marker::F64 => {
                let value = self.data.read_f64()?;
                self.deserialize_f64(value).map(Bound::into_any)
            }
            Marker::FixStr(len) => self.deserialize_str(len.into()).map(Bound::into_any),
            Marker::Str8 => {
                let len = self.data.read_u8()?;
                self.deserialize_str(len.into()).map(Bound::into_any)
            }
            Marker::Str16 => {
                let len = self.data.read_u16()?;
                self.deserialize_str(len.into()).map(Bound::into_any)
            }
            Marker::Str32 => {
                let len = self.data.read_u32()?;
                self.deserialize_str(len).map(Bound::into_any)
            }
            Marker::Bin8 => {
                let len = self.data.read_u8()?;
                self.deserialize_bin(len.into()).map(Bound::into_any)
            }
            Marker::Bin16 => {
                let len = self.data.read_u16()?;
                self.deserialize_bin(len.into()).map(Bound::into_any)
            }
            Marker::Bin32 => {
                let len = self.data.read_u32()?;
                self.deserialize_bin(len).map(Bound::into_any)
            }
            Marker::FixArray(len) => self.deserialize_array(len.into()).map(Bound::into_any),
            Marker::Array16 => {
                let len = self.data.read_u16()?;
                self.deserialize_array(len.into()).map(Bound::into_any)
            }
            Marker::Array32 => {
                let len = self.data.read_u32()?;
                self.deserialize_array(len).map(Bound::into_any)
            }
            Marker::FixMap(len) => self.deserialize_map(len.into()).map(Bound::into_any),
            Marker::Map16 => {
                let len = self.data.read_u16()?;
                self.deserialize_map(len.into()).map(Bound::into_any)
            }
            Marker::Map32 => {
                let len = self.data.read_u32()?;
                self.deserialize_map(len).map(Bound::into_any)
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

    fn deserialize_map_str_key(&mut self, len: u32) -> Result<Bound<'py, PyString>, Error> {
        if unlikely(len > 64) {
            let value = self.deserialize_str(len)?;
            hash_str(value.as_borrowed());
            Ok(value)
        } else {
            let data = self.data.read_slice(len as usize)?;
            Ok(self.state.key_map.get(self.py, data)?)
        }
    }

    fn deserialize_map_array_key(&mut self, len: u32) -> Result<Bound<'py, PyTuple>, Error> {
        let py = self.py;
        let state = self.state;
        let elements = (0..len).map(|_| {
            PyItem(
                self.deserialize_map_key()
                    .map_err(|e| e.into_pyerr(py, state)),
            )
        });
        PyTuple::new(py, elements).map_err(Error::PyErr)
    }

    fn deserialize_map_ext_key(&mut self, len: u32) -> Result<Bound<'py, PyDateTime>, Error> {
        let tag = self.data.read_i8()?;
        if tag == -1 && self.opts & DATETIME_AS_TIMESTAMP_EXT != 0 {
            self.deserialize_timestamp_ext(len)
        } else {
            Err(Error::InvalidValue)
        }
    }

    fn deserialize_map_key(&mut self) -> Result<Bound<'py, PyAny>, Error> {
        self.recursion += 1;
        if unlikely(self.recursion == RECURSION_LIMIT) {
            return Err(Error::RecursionLimitReached);
        }

        let marker = self.read_marker()?;
        let value = match marker {
            Marker::Null => self.deserialize_null().map(Bound::into_any),
            Marker::True => self.deserialize_true().map(Bound::into_any),
            Marker::False => self.deserialize_false().map(Bound::into_any),
            Marker::FixPos(value) => self.deserialize_u64(value.into()).map(Bound::into_any),
            Marker::U8 => {
                let value = self.data.read_u8()?;
                self.deserialize_u64(value.into()).map(Bound::into_any)
            }
            Marker::U16 => {
                let value = self.data.read_u16()?;
                self.deserialize_u64(value.into()).map(Bound::into_any)
            }
            Marker::U32 => {
                let value = self.data.read_u32()?;
                self.deserialize_u64(value.into()).map(Bound::into_any)
            }
            Marker::U64 => {
                let value = self.data.read_u64()?;
                self.deserialize_u64(value).map(Bound::into_any)
            }
            Marker::FixNeg(value) => self.deserialize_i64(value.into()).map(Bound::into_any),
            Marker::I8 => {
                let value = self.data.read_i8()?;
                self.deserialize_i64(value.into()).map(Bound::into_any)
            }
            Marker::I16 => {
                let value = self.data.read_i16()?;
                self.deserialize_i64(value.into()).map(Bound::into_any)
            }
            Marker::I32 => {
                let value = self.data.read_i32()?;
                self.deserialize_i64(value.into()).map(Bound::into_any)
            }
            Marker::I64 => {
                let value = self.data.read_i64()?;
                self.deserialize_i64(value).map(Bound::into_any)
            }
            Marker::F32 => {
                let value = self.data.read_f32()?;
                self.deserialize_f64(value.into()).map(Bound::into_any)
            }
            Marker::F64 => {
                let value = self.data.read_f64()?;
                self.deserialize_f64(value).map(Bound::into_any)
            }
            Marker::FixStr(len) => self
                .deserialize_map_str_key(len.into())
                .map(Bound::into_any),
            Marker::Str8 => {
                let len = self.data.read_u8()?;
                self.deserialize_map_str_key(len.into())
                    .map(Bound::into_any)
            }
            Marker::Str16 => {
                let len = self.data.read_u16()?;
                self.deserialize_map_str_key(len.into())
                    .map(Bound::into_any)
            }
            Marker::Str32 => {
                let len = self.data.read_u32()?;
                self.deserialize_map_str_key(len).map(Bound::into_any)
            }
            Marker::Bin8 => {
                let len = self.data.read_u8()?;
                self.deserialize_bin(len.into()).map(Bound::into_any)
            }
            Marker::Bin16 => {
                let len = self.data.read_u16()?;
                self.deserialize_bin(len.into()).map(Bound::into_any)
            }
            Marker::Bin32 => {
                let len = self.data.read_u32()?;
                self.deserialize_bin(len).map(Bound::into_any)
            }
            Marker::FixArray(len) => self
                .deserialize_map_array_key(len.into())
                .map(Bound::into_any),
            Marker::Array16 => {
                let len = self.data.read_u16()?;
                self.deserialize_map_array_key(len.into())
                    .map(Bound::into_any)
            }
            Marker::Array32 => {
                let len = self.data.read_u32()?;
                self.deserialize_map_array_key(len).map(Bound::into_any)
            }
            Marker::FixExt4 => self.deserialize_map_ext_key(4).map(Bound::into_any),
            Marker::FixExt8 => self.deserialize_map_ext_key(8).map(Bound::into_any),
            Marker::Ext8 => {
                let len = self.data.read_u8()?;
                self.deserialize_map_ext_key(len.into())
                    .map(Bound::into_any)
            }
            marker => Err(Error::InvalidType(marker)),
        };

        self.recursion -= 1;
        value
    }
}
