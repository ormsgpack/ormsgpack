// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
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
use crate::serialize::list::*;
use crate::serialize::memoryview::*;
use crate::serialize::numpy::*;
use crate::serialize::pydantic::*;
use crate::serialize::state::State;
use crate::serialize::str::*;
use crate::serialize::tuple::*;
use crate::serialize::uuid::*;
use crate::serialize::writer::*;
use crate::serialize::{pyerr_to_serde, Context, DictKeyContext};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use serde::ser::{Serialize, Serializer};

pub fn serialize<'a, 'py>(
    obj: Borrowed<'a, 'py, PyAny>,
    state: &'a State,
    default: Option<Borrowed<'a, 'py, PyAny>>,
    opts: Opt,
) -> PyResult<Bound<'py, PyBytes>> {
    let mut buf = BytesWriter::default();
    let default_hook = DefaultHook::new(default);
    let mut ser = msgpack::Serializer::new(&mut buf);
    let context = Context {
        state: state,
        opts: opts,
        default: &default_hook,
    };
    let res = PyObject::new(obj, context).serialize(&mut ser);
    match res {
        Ok(_) => Ok(buf.finish(obj.py())),
        Err(err) => Err(state.error(obj.py(), &err.to_string())),
    }
}

pub struct PyObject<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
    context: Context<'a, 'py>,
}

impl<'a, 'py> PyObject<'a, 'py> {
    pub fn new(obj: Borrowed<'a, 'py, PyAny>, context: Context<'a, 'py>) -> Self {
        Self {
            obj: obj,
            context: context,
        }
    }

    fn serialize_with_default_hook<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj = self
            .context
            .default
            .enter_call(self.obj)
            .map_err(serde::ser::Error::custom)?;
        let res = PyObject::new(obj.as_borrowed(), self.context).serialize(serializer);
        self.context.default.leave_call();
        res
    }

    #[inline(never)]
    fn serialize_unlikely<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj = BorrowedWithType::new(self.obj);
        let py = self.obj.py();

        if self.context.opts & PASSTHROUGH_DATETIME == 0 {
            match DateTime::try_new(obj, &self.context.state.datetime, self.context.opts) {
                Ok(Some(value)) => return value.serialize(serializer),
                Ok(None) => {}
                Err(err) => return Err(serde::ser::Error::custom(err)),
            }
            if let Some(value) = Date::try_new(obj, &self.context.state.datetime) {
                return value.serialize(serializer);
            }
            match Time::try_new(obj, &self.context.state.datetime, self.context.opts) {
                Ok(Some(value)) => return value.serialize(serializer),
                Ok(None) => {}
                Err(err) => return Err(serde::ser::Error::custom(err)),
            }
        }

        if self.context.opts & PASSTHROUGH_TUPLE == 0 {
            if let Some(value) = Tuple::try_new(obj, self.context) {
                return value.serialize(serializer);
            }
        }

        if self.context.opts & PASSTHROUGH_UUID == 0 {
            if let Some(value) = UUID::try_new(obj, &self.context.state.uuid) {
                return value.serialize(serializer);
            }
        }

        if let Some(value) = Enum::try_new(obj, &self.context.state.enum_, self.context) {
            if self.context.opts & PASSTHROUGH_ENUM == 0 {
                return value.serialize(serializer);
            } else {
                return self.serialize_with_default_hook(serializer);
            }
        }

        if self.context.opts & PASSTHROUGH_SUBCLASS == 0 {
            if let Some(value) = StrSubclass::try_new(obj, self.context.opts) {
                return value.serialize(serializer);
            }
            match Int::try_new(obj) {
                Ok(Some(value)) => return value.serialize(serializer),
                Ok(None) => {}
                Err(err) => {
                    if self.context.opts & PASSTHROUGH_BIG_INT != 0 {
                        return self.serialize_with_default_hook(serializer);
                    } else {
                        return Err(serde::ser::Error::custom(err));
                    }
                }
            }
            if let Some(value) = List::try_new(obj, self.context) {
                return value.serialize(serializer);
            }
            if let Some(value) = Dict::try_new(obj, self.context) {
                return value.serialize(serializer);
            }
        }

        if let Some(value) = Ext::try_new(obj, &self.context.state.ext) {
            return value.serialize(serializer);
        }

        if self.context.opts & PASSTHROUGH_DATACLASS == 0 {
            if let Some(value) = Dataclass::try_new(obj, self.context) {
                return value.serialize(serializer);
            }
        }

        if self.context.opts & SERIALIZE_PYDANTIC != 0 {
            if let Some(value) = PydanticModel::try_new(obj, self.context) {
                return value.serialize(serializer);
            }
        }

        if self.context.opts & SERIALIZE_NUMPY != 0 {
            if let Some(numpy_types_ref) = self
                .context
                .state
                .numpy
                .get_types(py)
                .map_err(|e| pyerr_to_serde(py, e))?
            {
                if let Some(value) = NumpyBool::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyDatetime64::try_new(
                    obj,
                    numpy_types_ref,
                    &self.context.state.numpy,
                    self.context.opts,
                ) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyFloat16::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyFloat32::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyFloat64::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyInt8::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyInt16::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyInt32::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyInt64::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyUint8::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyUint16::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyUint32::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                if let Some(value) = NumpyUint64::try_new(obj, numpy_types_ref) {
                    return value.serialize(serializer);
                }
                match NumpyArray::try_new(
                    obj,
                    numpy_types_ref,
                    &self.context.state.numpy,
                    self.context.opts,
                ) {
                    Ok(Some(value)) => return value.serialize(serializer),
                    Ok(None) => {}
                    Err(PyArrayError::Malformed) => {
                        return Err(serde::ser::Error::custom("numpy array is malformed"))
                    }
                    Err(PyArrayError::NotContiguous) | Err(PyArrayError::UnsupportedDataType) => {
                        if self.context.default.inner.is_none() {
                            return Err(serde::ser::Error::custom(
                                "numpy array is not C contiguous; use ndarray.tolist() in default",
                            ));
                        }
                    }
                }
            }
        }

        if let Some(value) = ByteArray::try_new(obj) {
            return value.serialize(serializer);
        }

        if let Some(value) = MemoryView::try_new(obj) {
            return value.serialize(serializer);
        }

        if let Some(value) = Fragment::try_new(obj, &self.context.state.fragment) {
            return value.serialize(serializer);
        }

        self.serialize_with_default_hook(serializer)
    }
}

impl Serialize for PyObject<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj = BorrowedWithType::new(self.obj);

        if let Some(value) = Str::try_new(obj, self.context.opts) {
            return value.serialize(serializer);
        }
        if let Some(value) = Bytes::try_new(obj) {
            return value.serialize(serializer);
        }
        match Int::try_new_exact(obj) {
            Ok(Some(value)) => return value.serialize(serializer),
            Ok(None) => {}
            Err(err) => {
                if self.context.opts & PASSTHROUGH_BIG_INT != 0 {
                    return self.serialize_with_default_hook(serializer);
                } else {
                    return Err(serde::ser::Error::custom(err));
                }
            }
        }
        if let Some(value) = Bool::try_new(obj) {
            return value.serialize(serializer);
        }
        if obj.as_borrowed().is_none() {
            return serializer.serialize_unit();
        }
        if let Some(value) = Float::try_new(obj) {
            return value.serialize(serializer);
        }
        if let Some(value) = List::try_new_exact(obj, self.context) {
            return value.serialize(serializer);
        }
        if let Some(value) = Dict::try_new_exact(obj, self.context) {
            return value.serialize(serializer);
        }
        self.serialize_unlikely(serializer)
    }
}

pub struct DictKey<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
    context: DictKeyContext<'a>,
}

impl<'a, 'py> DictKey<'a, 'py> {
    pub fn new(obj: Borrowed<'a, 'py, PyAny>, context: DictKeyContext<'a>) -> Self {
        Self {
            obj: obj,
            context: context,
        }
    }

    #[inline(never)]
    fn serialize_unlikely<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj = BorrowedWithType::new(self.obj);

        match DateTime::try_new(obj, &self.context.state.datetime, self.context.opts) {
            Ok(Some(value)) => return value.serialize(serializer),
            Ok(None) => {}
            Err(err) => return Err(serde::ser::Error::custom(err)),
        }
        if let Some(value) = Date::try_new(obj, &self.context.state.datetime) {
            return value.serialize(serializer);
        }
        match Time::try_new(obj, &self.context.state.datetime, self.context.opts) {
            Ok(Some(value)) => return value.serialize(serializer),
            Ok(None) => {}
            Err(err) => return Err(serde::ser::Error::custom(err)),
        }

        if let Some(value) = Tuple::try_new(obj, self.context) {
            return value.serialize(serializer);
        }

        if let Some(value) = UUID::try_new(obj, &self.context.state.uuid) {
            return value.serialize(serializer);
        }

        if let Some(value) = Enum::try_new(obj, &self.context.state.enum_, self.context) {
            return value.serialize(serializer);
        }

        if let Some(value) = StrSubclass::try_new(obj, self.context.opts) {
            return value.serialize(serializer);
        }
        match Int::try_new(obj) {
            Ok(Some(value)) => return value.serialize(serializer),
            Ok(None) => {}
            Err(err) => return Err(serde::ser::Error::custom(err)),
        }

        if let Some(value) = MemoryView::try_new(obj) {
            return value.serialize(serializer);
        }

        Err(serde::ser::Error::custom(
            "Dict key must a type serializable with OPT_NON_STR_KEYS",
        ))
    }
}

impl Serialize for DictKey<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj = BorrowedWithType::new(self.obj);

        if let Some(value) = Str::try_new(obj, self.context.opts) {
            return value.serialize(serializer);
        }
        if let Some(value) = Bytes::try_new(obj) {
            return value.serialize(serializer);
        }
        match Int::try_new_exact(obj) {
            Ok(Some(value)) => return value.serialize(serializer),
            Ok(None) => {}
            Err(err) => return Err(serde::ser::Error::custom(err)),
        }
        if let Some(value) = Bool::try_new(obj) {
            return value.serialize(serializer);
        }
        if obj.as_borrowed().is_none() {
            return serializer.serialize_unit();
        }
        if let Some(value) = Float::try_new(obj) {
            return value.serialize(serializer);
        }
        self.serialize_unlikely(serializer)
    }
}
