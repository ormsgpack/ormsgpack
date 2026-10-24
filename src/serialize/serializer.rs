// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::ffi::*;
use crate::io::WriteSlices;
use crate::msgpack;
use crate::opt::*;
use crate::serialize::bool_::*;
use crate::serialize::bytearray::*;
use crate::serialize::bytes::*;
use crate::serialize::dataclass::*;
use crate::serialize::datetime::*;
use crate::serialize::default::*;
use crate::serialize::dict::*;
use crate::serialize::enum_::*;
use crate::serialize::ext::*;
use crate::serialize::float::*;
use crate::serialize::fragment::*;
#[cfg(Py_3_15)]
use crate::serialize::frozendict::*;
use crate::serialize::list::*;
use crate::serialize::memoryview::*;
use crate::serialize::numpy::*;
use crate::serialize::pydantic::*;
use crate::serialize::state::State;
use crate::serialize::str::*;
use crate::serialize::tuple::*;
use crate::serialize::uuid::*;
use crate::serialize::writer::*;
use crate::util::unlikely;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    PyErr(PyErr),
    Write,
}

impl From<std::io::Error> for Error {
    #[cold]
    fn from(_: std::io::Error) -> Error {
        Error::Write
    }
}

impl From<PyErr> for Error {
    #[cold]
    fn from(err: PyErr) -> Self {
        Error::PyErr(err)
    }
}

impl Error {
    #[cold]
    pub fn custom<T>(msg: T) -> Error
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }

    #[cold]
    pub fn into_pyerr(self, py: Python<'_>, state: &State) -> PyErr {
        match self {
            Self::Custom(message) => state.error(py, &message),
            Self::PyErr(err) => {
                let error = state.error(py, "serialize error");
                error.set_cause(py, Some(err));
                error
            }
            Self::Write => state.error(py, "write error"),
        }
    }
}

pub fn serialize<'a, 'py>(
    obj: Borrowed<'a, 'py, PyAny>,
    state: &'a State,
    default: Option<Borrowed<'a, 'py, PyAny>>,
    opts: Opt,
) -> PyResult<Bound<'py, PyBytes>> {
    let default_hook = DefaultHook::new(default);
    let mut writer = BytesWriter::default();
    let mut serializer = Serializer::new(obj.py(), &mut writer, state, &default_hook, opts);
    serializer
        .serialize(obj)
        .map_err(|e| e.into_pyerr(obj.py(), state))?;
    Ok(writer.finish(obj.py()))
}

pub struct Serializer<'a, 'py, W> {
    py: Python<'py>,
    writer: W,
    state: &'a State,
    default: &'a DefaultHook<'a, 'py>,
    opts: Opt,
    recursion: u8,
}

impl<'a, 'py, W: WriteSlices> Serializer<'a, 'py, W> {
    pub fn new(
        py: Python<'py>,
        writer: W,
        state: &'a State,
        default: &'a DefaultHook<'a, 'py>,
        opts: Opt,
    ) -> Self {
        Self {
            py,
            writer,
            state,
            default,
            opts,
            recursion: 0,
        }
    }

    fn serialize_with_default_hook(&mut self, obj: Borrowed<'_, '_, PyAny>) -> Result<(), Error> {
        let obj = self.default.enter_call(obj).map_err(Error::custom)?;
        let res = self.serialize(obj.as_borrowed());
        self.default.leave_call();
        res
    }

    #[inline(never)]
    fn serialize_unlikely(&mut self, obj: Borrowed<'_, '_, PyAny>) -> Result<(), Error> {
        let obj = BorrowedWithType::new(obj);

        if self.opts & PASSTHROUGH_DATETIME == 0 {
            match DateTime::try_new(obj, &self.state.datetime, self.opts) {
                Ok(Some(value)) => return value.serialize(self),
                Ok(None) => {}
                Err(err) => return Err(Error::custom(err)),
            }
            if let Some(value) = Date::try_new(obj, &self.state.datetime) {
                return value.serialize(self);
            }
            match Time::try_new(obj, &self.state.datetime, self.opts) {
                Ok(Some(value)) => return value.serialize(self),
                Ok(None) => {}
                Err(err) => return Err(Error::custom(err)),
            }
        }

        if self.opts & PASSTHROUGH_TUPLE == 0 {
            if let Some(value) = Tuple::try_new(obj) {
                return value.serialize(self);
            }
        }

        if self.opts & PASSTHROUGH_UUID == 0 {
            if let Some(value) = UUID::try_new(obj, &self.state.uuid) {
                return value.serialize(self);
            }
        }

        if let Some(value) = Enum::try_new(obj, &self.state.enum_) {
            if self.opts & PASSTHROUGH_ENUM == 0 {
                return value.serialize(self);
            } else {
                return self.serialize_with_default_hook(obj.as_borrowed());
            }
        }

        if self.opts & PASSTHROUGH_SUBCLASS == 0 {
            if let Some(value) = StrSubclass::try_new(obj, self.opts) {
                return value.serialize(self);
            }
            match Int::try_new(obj) {
                Ok(Some(value)) => return value.serialize(self),
                Ok(None) => {}
                Err(err) => {
                    if self.opts & PASSTHROUGH_BIG_INT != 0 {
                        return self.serialize_with_default_hook(obj.as_borrowed());
                    } else {
                        return Err(Error::custom(err));
                    }
                }
            }
            if let Some(value) = List::try_new(obj) {
                return value.serialize(self);
            }
            if let Some(value) = Dict::try_new(obj, self.opts) {
                return value.serialize(self);
            }
            #[cfg(Py_3_15)]
            if let Some(value) = FrozenDict::try_new(obj, self.opts) {
                return value.serialize(self);
            }
        }

        if let Some(value) = Ext::try_new(obj, &self.state.ext) {
            return value.serialize(self);
        }

        if self.opts & PASSTHROUGH_DATACLASS == 0 {
            if let Some(value) = Dataclass::try_new(obj, &self.state.dataclass) {
                return value.serialize(self);
            }
        }

        if self.opts & SERIALIZE_PYDANTIC != 0 {
            if let Some(value) = PydanticModel::try_new(obj, &self.state.pydantic, self.opts) {
                return value.serialize(self);
            }
        }

        if self.opts & SERIALIZE_NUMPY != 0 {
            if let Some(numpy_types_ref) = self.state.numpy.get_types(self.py)? {
                if let Some(value) = NumpyBool::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) =
                    NumpyDatetime64::try_new(obj, numpy_types_ref, &self.state.numpy, self.opts)
                {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyFloat16::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyFloat32::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyFloat64::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyInt8::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyInt16::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyInt32::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyInt64::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyUint8::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyUint16::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyUint32::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                if let Some(value) = NumpyUint64::try_new(obj, numpy_types_ref) {
                    return value.serialize(self);
                }
                match NumpyArray::try_new(obj, numpy_types_ref, &self.state.numpy, self.opts) {
                    Ok(Some(value)) => return value.serialize(self),
                    Ok(None) => {}
                    Err(PyArrayError::Malformed) => {
                        return Err(Error::custom("numpy array is malformed"))
                    }
                    Err(PyArrayError::NotContiguous) | Err(PyArrayError::UnsupportedDataType) => {
                        if self.default.inner.is_none() {
                            return Err(Error::custom(
                                "numpy array is not C contiguous; use ndarray.tolist() in default",
                            ));
                        }
                    }
                }
            }
        }

        if let Some(value) = ByteArray::try_new(obj) {
            return value.serialize(self);
        }

        if let Some(value) = MemoryView::try_new(obj) {
            return value.serialize(self);
        }

        if let Some(value) = Fragment::try_new(obj, &self.state.fragment) {
            return value.serialize(self);
        }

        #[cfg(Py_3_15)]
        if let Some(value) = FrozenDict::try_new_exact(obj, self.opts) {
            return value.serialize(self);
        }

        self.serialize_with_default_hook(obj.as_borrowed())
    }

    pub fn serialize(&mut self, obj: Borrowed<'_, '_, PyAny>) -> Result<(), Error> {
        let obj = BorrowedWithType::new(obj);

        if let Some(value) = Str::try_new(obj, self.opts) {
            return value.serialize(self);
        }
        if let Some(value) = Bytes::try_new(obj) {
            return value.serialize(self);
        }
        match Int::try_new_exact(obj) {
            Ok(Some(value)) => return value.serialize(self),
            Ok(None) => {}
            Err(err) => {
                if self.opts & PASSTHROUGH_BIG_INT != 0 {
                    return self.serialize_with_default_hook(obj.as_borrowed());
                } else {
                    return Err(Error::custom(err));
                }
            }
        }
        if let Some(value) = Bool::try_new(obj) {
            return value.serialize(self);
        }
        if obj.as_borrowed().is_none() {
            return self.serialize_unit();
        }
        if let Some(value) = Float::try_new(obj) {
            return value.serialize(self);
        }
        if let Some(value) = List::try_new_exact(obj) {
            return value.serialize(self);
        }
        if let Some(value) = Dict::try_new_exact(obj, self.opts) {
            return value.serialize(self);
        }
        self.serialize_unlikely(obj.as_borrowed())
    }

    #[inline(never)]
    fn serialize_key_unlikely(&mut self, obj: Borrowed<'_, '_, PyAny>) -> Result<(), Error> {
        let obj = BorrowedWithType::new(obj);

        match DateTime::try_new(obj, &self.state.datetime, self.opts) {
            Ok(Some(value)) => return value.serialize_key(self),
            Ok(None) => {}
            Err(err) => return Err(Error::custom(err)),
        }
        if let Some(value) = Date::try_new(obj, &self.state.datetime) {
            return value.serialize_key(self);
        }
        match Time::try_new(obj, &self.state.datetime, self.opts) {
            Ok(Some(value)) => return value.serialize_key(self),
            Ok(None) => {}
            Err(err) => return Err(Error::custom(err)),
        }

        if let Some(value) = Tuple::try_new(obj) {
            return value.serialize_key(self);
        }

        if let Some(value) = UUID::try_new(obj, &self.state.uuid) {
            return value.serialize_key(self);
        }

        if let Some(value) = Enum::try_new(obj, &self.state.enum_) {
            return value.serialize_key(self);
        }

        if let Some(value) = StrSubclass::try_new(obj, self.opts) {
            return value.serialize_key(self);
        }
        match Int::try_new(obj) {
            Ok(Some(value)) => return value.serialize_key(self),
            Ok(None) => {}
            Err(err) => return Err(Error::custom(err)),
        }

        if let Some(value) = MemoryView::try_new(obj) {
            return value.serialize_key(self);
        }

        Err(Error::custom(
            "Dict key must a type serializable with OPT_NON_STR_KEYS",
        ))
    }

    pub fn serialize_key(&mut self, obj: Borrowed<'_, '_, PyAny>) -> Result<(), Error> {
        let obj = BorrowedWithType::new(obj);

        if let Some(value) = Str::try_new(obj, self.opts) {
            return value.serialize_key(self);
        }
        if let Some(value) = Bytes::try_new(obj) {
            return value.serialize_key(self);
        }
        match Int::try_new_exact(obj) {
            Ok(Some(value)) => return value.serialize_key(self),
            Ok(None) => {}
            Err(err) => return Err(Error::custom(err)),
        }
        if let Some(value) = Bool::try_new(obj) {
            return value.serialize_key(self);
        }
        if obj.as_borrowed().is_none() {
            return self.serialize_unit();
        }
        if let Some(value) = Float::try_new(obj) {
            return value.serialize_key(self);
        }
        self.serialize_key_unlikely(obj.as_borrowed())
    }

    pub fn serialize_bool(&mut self, value: bool) -> Result<(), Error> {
        msgpack::write_bool(&mut self.writer, value)?;
        Ok(())
    }

    pub fn serialize_f32(&mut self, value: f32) -> Result<(), Error> {
        msgpack::write_f32(&mut self.writer, value)?;
        Ok(())
    }

    pub fn serialize_f64(&mut self, value: f64) -> Result<(), Error> {
        msgpack::write_f64(&mut self.writer, value)?;
        Ok(())
    }

    pub fn serialize_i64(&mut self, value: i64) -> Result<(), Error> {
        msgpack::write_i64(&mut self.writer, value)?;
        Ok(())
    }

    pub fn serialize_u64(&mut self, value: u64) -> Result<(), Error> {
        msgpack::write_u64(&mut self.writer, value)?;
        Ok(())
    }

    pub fn serialize_str(&mut self, value: &str) -> Result<(), Error> {
        msgpack::write_str(&mut self.writer, value)?;
        Ok(())
    }

    pub fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Error> {
        msgpack::write_bin(&mut self.writer, value)?;
        Ok(())
    }

    pub fn serialize_unit(&mut self) -> Result<(), Error> {
        msgpack::write_nil(&mut self.writer)?;
        Ok(())
    }

    pub fn serialize_ext(&mut self, tag: i8, value: &[u8]) -> Result<(), Error> {
        msgpack::write_ext(&mut self.writer, value, tag)?;
        Ok(())
    }

    pub fn serialize_fragment(&mut self, value: &[u8]) -> Result<(), Error> {
        self.writer.write_all(value)?;
        Ok(())
    }

    pub fn serialize_seq<F>(&mut self, len: usize, serialize: F) -> Result<(), Error>
    where
        F: FnOnce(&mut Sequence<'_, 'a, 'py, W>) -> Result<(), Error>,
    {
        if unlikely(self.recursion == msgpack::RECURSION_LIMIT) {
            return Err(Error::custom(RECURSION_LIMIT_REACHED));
        }
        msgpack::write_array_len(&mut self.writer, len)?;
        self.recursion += 1;
        let result = serialize(&mut Sequence { serializer: self });
        self.recursion -= 1;
        result
    }

    pub fn serialize_map<F>(&mut self, len: usize, serialize: F) -> Result<(), Error>
    where
        F: FnOnce(&mut Map<'_, 'a, 'py, W>) -> Result<(), Error>,
    {
        if unlikely(self.recursion == msgpack::RECURSION_LIMIT) {
            return Err(Error::custom(RECURSION_LIMIT_REACHED));
        }
        msgpack::write_map_len(&mut self.writer, len)?;
        self.recursion += 1;
        let result = serialize(&mut Map { serializer: self });
        self.recursion -= 1;
        result
    }
}

pub struct Sequence<'s, 'a, 'py, W> {
    serializer: &'s mut Serializer<'a, 'py, W>,
}

impl<W: WriteSlices> Sequence<'_, '_, '_, W> {
    pub fn serialize_element_as<const KEY: bool, T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize_as::<KEY, W>(self.serializer)
    }

    pub fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.serialize_element_as::<false, T>(value)
    }
}

pub struct Map<'s, 'a, 'py, W> {
    serializer: &'s mut Serializer<'a, 'py, W>,
}

impl<W: WriteSlices> Map<'_, '_, '_, W> {
    pub fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Error> {
        key.serialize_key(self.serializer)?;
        value.serialize(self.serializer)
    }
}

pub trait Serialize {
    fn serialize_as<const KEY: bool, W: WriteSlices>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>;

    fn serialize<W: WriteSlices>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error> {
        self.serialize_as::<false, W>(serializer)
    }

    fn serialize_key<W: WriteSlices>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error> {
        self.serialize_as::<true, W>(serializer)
    }
}

impl<T: ?Sized + Serialize> Serialize for &T {
    fn serialize_as<const KEY: bool, W: WriteSlices>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error> {
        (**self).serialize_as::<KEY, W>(serializer)
    }
}

impl Serialize for Borrowed<'_, '_, PyAny> {
    fn serialize_as<const KEY: bool, W: WriteSlices>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error> {
        if KEY {
            serializer.serialize_key(*self)
        } else {
            serializer.serialize(*self)
        }
    }
}

impl Serialize for Int {
    #[inline]
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        match self {
            Int::Signed(value) => serializer.serialize_i64(*value),
            Int::Unsigned(value) => serializer.serialize_u64(*value),
        }
    }
}

impl Serialize for str {
    fn serialize_as<const KEY: bool, W: WriteSlices>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error> {
        serializer.serialize_str(self)
    }
}

macro_rules! impl_serialize {
    ($($ty:ty => $method:ident),* $(,)?) => {
        $(impl Serialize for $ty {
            fn serialize_as<const KEY: bool, W: WriteSlices>(&self, serializer: &mut Serializer<'_, '_, W>) -> Result<(), Error> {
                serializer.$method((*self).into())
            }
        })*
    };
}

impl_serialize! {
    bool => serialize_bool,
    f32 => serialize_f32,
    f64 => serialize_f64,
    i8 => serialize_i64,
    i16 => serialize_i64,
    i32 => serialize_i64,
    i64 => serialize_i64,
    u8 => serialize_u64,
    u16 => serialize_u64,
    u32 => serialize_u64,
    u64 => serialize_u64,
}
