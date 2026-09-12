use crate::ffi::*;
use crate::opt::*;
use crate::serialize::datetimelike::NaiveDateTime;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta};
use pyo3::ffi::{PyObject, Py_intptr_t};
use pyo3::prelude::*;
use pyo3::types::{PyCapsule, PyList, PyString, PyTuple};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::OnceLock;

// https://numpy.org/doc/1.26/reference/arrays.interface.html#object.__array_struct__

#[repr(C)]
pub struct PyArrayInterface {
    pub two: c_int,
    pub nd: c_int,
    pub typekind: c_char,
    pub itemsize: c_int,
    pub flags: c_int,
    pub shape: *mut Py_intptr_t,
    pub strides: *mut Py_intptr_t,
    pub data: *mut c_void,
    pub descr: *mut PyObject,
}

pub struct NumpyTypes {
    pub bool_: Py<PyAny>,
    pub datetime64: Py<PyAny>,
    pub float16: Py<PyAny>,
    pub float32: Py<PyAny>,
    pub float64: Py<PyAny>,
    pub int8: Py<PyAny>,
    pub int16: Py<PyAny>,
    pub int32: Py<PyAny>,
    pub int64: Py<PyAny>,
    pub uint8: Py<PyAny>,
    pub uint16: Py<PyAny>,
    pub uint32: Py<PyAny>,
    pub uint64: Py<PyAny>,
    pub ndarray: Py<PyAny>,
}

impl NumpyTypes {
    #[cold]
    fn load(py: Python<'_>) -> PyResult<Option<Self>> {
        let numpy = match py.import("numpy") {
            Ok(module) => module,
            Err(_) => return Ok(None),
        };

        Ok(Some(Self {
            bool_: numpy.getattr("bool_")?.unbind(),
            datetime64: numpy.getattr("datetime64")?.unbind(),
            float16: numpy.getattr("half")?.unbind(),
            float32: numpy.getattr("float32")?.unbind(),
            float64: numpy.getattr("float64")?.unbind(),
            int8: numpy.getattr("int8")?.unbind(),
            int16: numpy.getattr("int16")?.unbind(),
            int32: numpy.getattr("int32")?.unbind(),
            int64: numpy.getattr("int64")?.unbind(),
            uint8: numpy.getattr("uint8")?.unbind(),
            uint16: numpy.getattr("uint16")?.unbind(),
            uint32: numpy.getattr("uint32")?.unbind(),
            uint64: numpy.getattr("uint64")?.unbind(),
            ndarray: numpy.getattr("ndarray")?.unbind(),
        }))
    }
}

pub struct State {
    types: OnceLock<Option<NumpyTypes>>,
    array_struct_str: Py<PyString>,
    descr_str: Py<PyString>,
    dtype_str: Py<PyString>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> Self {
        Self {
            types: OnceLock::new(),
            array_struct_str: PyString::intern(py, "__array_struct__").unbind(),
            descr_str: PyString::intern(py, "descr").unbind(),
            dtype_str: PyString::intern(py, "dtype").unbind(),
        }
    }

    pub fn get_types(&self, py: Python<'_>) -> PyResult<&Option<NumpyTypes>> {
        if self.types.get().is_none() {
            let types = NumpyTypes::load(py)?;
            let _ = self.types.set(types);
        }
        Ok(self.types.get().unwrap())
    }
}

/// Get the dtype description of a numpy scalar or array.
///
/// We cannot use the `descr` field of `__array_struct__` because numpy does
/// not populate it for datetime64 arrays; see
/// https://github.com/numpy/numpy/issues/5350.
fn get_dtype_descr<'py>(
    obj: Borrowed<'_, 'py, PyAny>,
    state: &State,
) -> Option<Bound<'py, PyString>> {
    let dtype = obj.getattr(state.dtype_str.bind_borrowed(obj.py())).ok()?;
    let descr = dtype
        .getattr(state.descr_str.bind_borrowed(obj.py()))
        .ok()
        .and_then(cast_into_exact::<PyList>)?;
    let item = descr
        .get_item(0)
        .ok()
        .and_then(cast_into_exact::<PyTuple>)?;
    item.get_item(1).ok().and_then(cast_into_exact::<PyString>)
}

#[derive(Clone, Copy)]
enum ItemType {
    BOOL,
    DATETIME64(NumpyDatetimeUnit),
    F16,
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl ItemType {
    fn find(
        array: *mut PyArrayInterface,
        obj: Borrowed<'_, '_, PyAny>,
        state: &State,
    ) -> Option<ItemType> {
        match unsafe { ((*array).typekind, (*array).itemsize) } {
            (098, 1) => Some(ItemType::BOOL),
            (077, 8) => {
                let descr = get_dtype_descr(obj, state)?;
                let unit = NumpyDatetimeUnit::from_str(descr.as_borrowed());
                Some(ItemType::DATETIME64(unit))
            }
            (102, 2) => Some(ItemType::F16),
            (102, 4) => Some(ItemType::F32),
            (102, 8) => Some(ItemType::F64),
            (105, 1) => Some(ItemType::I8),
            (105, 2) => Some(ItemType::I16),
            (105, 4) => Some(ItemType::I32),
            (105, 8) => Some(ItemType::I64),
            (117, 1) => Some(ItemType::U8),
            (117, 2) => Some(ItemType::U16),
            (117, 4) => Some(ItemType::U32),
            (117, 8) => Some(ItemType::U64),
            _ => None,
        }
    }
}

pub enum PyArrayError {
    Malformed,
    NotContiguous,
    UnsupportedDataType,
}

struct NumpyArrayData {
    data: *const c_void,
    len: usize,
    kind: ItemType,
    opts: Opt,
}

impl Serialize for NumpyArrayData {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len))?;
        match self.kind {
            ItemType::BOOL => {
                let slice: &[u8] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<u8>(), self.len) };
                for &each in slice.iter() {
                    let value = each == 1;
                    seq.serialize_element(&value).unwrap();
                }
            }
            ItemType::DATETIME64(unit) => {
                let slice: &[i64] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<i64>(), self.len) };
                let convert = unit.converter().map_err(serde::ser::Error::custom)?;
                for &each in slice.iter() {
                    let value = convert(each, self.opts).map_err(serde::ser::Error::custom)?;
                    seq.serialize_element(&value).unwrap();
                }
            }
            ItemType::F16 => {
                let slice: &[u16] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<u16>(), self.len) };
                for &each in slice.iter() {
                    let value = half::f16::from_bits(each).to_f32();
                    seq.serialize_element(&value).unwrap();
                }
            }
            ItemType::F32 => {
                let slice: &[f32] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<f32>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
            ItemType::F64 => {
                let slice: &[f64] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<f64>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
            ItemType::I8 => {
                let slice: &[i8] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<i8>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
            ItemType::I16 => {
                let slice: &[i16] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<i16>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
            ItemType::I32 => {
                let slice: &[i32] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<i32>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
            ItemType::I64 => {
                let slice: &[i64] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<i64>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
            ItemType::U8 => {
                let slice: &[u8] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<u8>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
            ItemType::U16 => {
                let slice: &[u16] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<u16>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
            ItemType::U32 => {
                let slice: &[u32] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<u32>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
            ItemType::U64 => {
                let slice: &[u64] =
                    unsafe { std::slice::from_raw_parts(self.data.cast::<u64>(), self.len) };
                for &each in slice.iter() {
                    seq.serialize_element(&each).unwrap();
                }
            }
        }
        seq.end()
    }
}

enum NumpyArrayNode {
    Internal(Vec<NumpyArrayNode>),
    Leaf(NumpyArrayData),
}

impl Serialize for NumpyArrayNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Internal(children) => {
                let mut seq = serializer.serialize_seq(Some(children.len()))?;
                for child in children {
                    seq.serialize_element(child).unwrap();
                }
                seq.end()
            }
            Self::Leaf(value) => value.serialize(serializer),
        }
    }
}

// >>> arr = numpy.array([[[1, 2], [3, 4]], [[5, 6], [7, 8]]], numpy.int32)
// >>> arr.ndim
// 3
// >>> arr.shape
// (2, 2, 2)
// >>> arr.strides
// (16, 8, 4)
pub struct NumpyArray<'py> {
    _capsule: Bound<'py, PyAny>,
    root: NumpyArrayNode,
}

impl<'py> NumpyArray<'py> {
    #[inline]
    pub fn try_new(
        obj: BorrowedWithType<'_, 'py>,
        types: &NumpyTypes,
        state: &State,
        opts: Opt,
    ) -> Result<Option<Self>, PyArrayError> {
        if obj.get_type_ptr() == types.ndarray.as_ptr().cast() {
            Self::new(obj.as_borrowed(), state, opts).map(Some)
        } else {
            Ok(None)
        }
    }

    #[inline(never)]
    fn new(obj: Borrowed<'_, 'py, PyAny>, state: &State, opts: Opt) -> Result<Self, PyArrayError> {
        unsafe {
            let capsule = obj
                .getattr(state.array_struct_str.bind_borrowed(obj.py()))
                .unwrap();
            let array = capsule
                .cast_unchecked::<PyCapsule>()
                .pointer_checked(None)
                .unwrap()
                .cast::<PyArrayInterface>()
                .as_ptr();
            if (*array).two != 2 {
                return Err(PyArrayError::Malformed);
            }
            if (*array).flags & 0x1 != 0x1 {
                return Err(PyArrayError::NotContiguous);
            }
            let num_dimensions = (*array).nd as usize;
            if num_dimensions == 0 {
                return Err(PyArrayError::UnsupportedDataType);
            }
            match ItemType::find(array, obj, state) {
                None => Err(PyArrayError::UnsupportedDataType),
                Some(kind) => {
                    let root = if num_dimensions > 1 {
                        let mut position = Vec::with_capacity(num_dimensions);
                        NumpyArray::build(array, kind, opts, 0, &mut position)
                    } else {
                        let shape = std::slice::from_raw_parts(
                            (*array).shape.cast::<isize>(),
                            num_dimensions,
                        );
                        NumpyArrayNode::Leaf(NumpyArrayData {
                            data: (*array).data,
                            len: shape[0] as usize,
                            kind: kind,
                            opts: opts,
                        })
                    };
                    Ok(NumpyArray {
                        _capsule: capsule,
                        root: root,
                    })
                }
            }
        }
    }

    fn build(
        array: *mut PyArrayInterface,
        kind: ItemType,
        opts: Opt,
        depth: usize,
        position: &mut Vec<isize>,
    ) -> NumpyArrayNode {
        let num_dimensions = unsafe { (*array).nd as usize };
        let shape =
            unsafe { std::slice::from_raw_parts((*array).shape.cast::<isize>(), num_dimensions) };
        let strides =
            unsafe { std::slice::from_raw_parts((*array).strides.cast::<isize>(), num_dimensions) };
        let num_children = shape[depth];
        let mut children = Vec::with_capacity(num_children as usize);
        for i in 0..num_children {
            position.push(i);
            let child = if depth < num_dimensions - 2 {
                NumpyArray::build(array, kind, opts, depth + 1, position)
            } else {
                let offset = strides
                    .iter()
                    .zip(position.iter())
                    .map(|(a, b)| a * b)
                    .sum::<isize>();
                NumpyArrayNode::Leaf(NumpyArrayData {
                    data: unsafe { (*array).data.offset(offset) },
                    len: shape[num_dimensions - 1] as usize,
                    kind: kind,
                    opts: opts,
                })
            };
            position.pop();
            children.push(child);
        }
        NumpyArrayNode::Internal(children)
    }
}

impl Serialize for NumpyArray<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.root.serialize(serializer)
    }
}

/// This mimicks the units supported by numpy's datetime64 type.
///
/// See
/// https://github.com/numpy/numpy/blob/v1.26.4/numpy/core/include/numpy/ndarraytypes.h#L244-L258
#[derive(Clone, Copy)]
enum NumpyDatetimeUnit {
    NaT,
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Picoseconds,
    Femtoseconds,
    Attoseconds,
    Generic,
}

impl std::fmt::Display for NumpyDatetimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unit = match self {
            Self::NaT => "NaT",
            Self::Years => "years",
            Self::Months => "months",
            Self::Weeks => "weeks",
            Self::Days => "days",
            Self::Hours => "hours",
            Self::Minutes => "minutes",
            Self::Seconds => "seconds",
            Self::Milliseconds => "milliseconds",
            Self::Microseconds => "microseconds",
            Self::Nanoseconds => "nanoseconds",
            Self::Picoseconds => "picoseconds",
            Self::Femtoseconds => "femtoseconds",
            Self::Attoseconds => "attoseconds",
            Self::Generic => "generic",
        };
        write!(f, "{unit}")
    }
}

enum NumpyDateTimeError {
    UnsupportedUnit(NumpyDatetimeUnit),
    Unrepresentable { unit: NumpyDatetimeUnit, val: i64 },
    NaT,
}

impl std::fmt::Display for NumpyDateTimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedUnit(unit) => write!(f, "unsupported numpy.datetime64 unit: {unit}"),
            Self::Unrepresentable { unit, val } => {
                write!(f, "unrepresentable numpy.datetime64: {val} {unit}")
            }
            Self::NaT => write!(f, "unrepresentable numpy.datetime64: NaT"),
        }
    }
}

type NumpyDatetimeConverter = fn(i64, Opt) -> Result<NaiveDateTime, NumpyDateTimeError>;

impl NumpyDatetimeUnit {
    fn from_str(obj: Borrowed<'_, '_, PyString>) -> Self {
        let uni = unicode_to_str(obj).unwrap();

        if uni.len() < 5 {
            return Self::NaT;
        }
        // unit descriptions are found at
        // https://github.com/numpy/numpy/blob/v1.26.4/numpy/core/src/multiarray/datetime.c#L81-L98
        match &uni[4..uni.len() - 1] {
            "Y" => Self::Years,
            "M" => Self::Months,
            "W" => Self::Weeks,
            "D" => Self::Days,
            "h" => Self::Hours,
            "m" => Self::Minutes,
            "s" => Self::Seconds,
            "ms" => Self::Milliseconds,
            "us" => Self::Microseconds,
            "ns" => Self::Nanoseconds,
            "ps" => Self::Picoseconds,
            "fs" => Self::Femtoseconds,
            "as" => Self::Attoseconds,
            "generic" => Self::Generic,
            _ => unreachable!(),
        }
    }

    /// Return a `NaiveDateTime` for a value with this unit.
    ///
    /// Returns an `Err(NumpyDateTimeError)` if the value is invalid for this unit.
    fn datetime(&self, val: i64, opts: Opt) -> Result<NaiveDateTime, NumpyDateTimeError> {
        self.converter()?(val, opts)
    }

    fn check_nat(val: i64) -> Result<(), NumpyDateTimeError> {
        if val == i64::MIN {
            Err(NumpyDateTimeError::NaT)
        } else {
            Ok(())
        }
    }

    fn datetime_from_years(val: i64, opts: Opt) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = NaiveDate::from_ymd_opt(
            (val + 1970)
                .try_into()
                .map_err(|_| NumpyDateTimeError::Unrepresentable {
                    unit: Self::Years,
                    val,
                })?,
            1,
            1,
        )
        .ok_or(NumpyDateTimeError::Unrepresentable {
            unit: Self::Years,
            val,
        })?
        .and_time(NaiveTime::MIN);
        Ok(NaiveDateTime { dt, opts })
    }

    fn datetime_from_months(val: i64, opts: Opt) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = NaiveDate::from_ymd_opt(
            (val.div_euclid(12) + 1970).try_into().map_err(|_| {
                NumpyDateTimeError::Unrepresentable {
                    unit: Self::Months,
                    val,
                }
            })?,
            (val.rem_euclid(12) + 1).try_into().map_err(|_| {
                NumpyDateTimeError::Unrepresentable {
                    unit: Self::Months,
                    val,
                }
            })?,
            1,
        )
        .ok_or(NumpyDateTimeError::Unrepresentable {
            unit: Self::Months,
            val,
        })?
        .and_time(NaiveTime::MIN);
        Ok(NaiveDateTime { dt, opts })
    }

    fn datetime_from_weeks(val: i64, opts: Opt) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = TimeDelta::try_weeks(val)
            .and_then(|delta| DateTime::UNIX_EPOCH.checked_add_signed(delta))
            .ok_or(NumpyDateTimeError::Unrepresentable {
                unit: Self::Weeks,
                val,
            })?
            .naive_utc();
        Ok(NaiveDateTime { dt, opts })
    }

    fn datetime_from_days(val: i64, opts: Opt) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = TimeDelta::try_days(val)
            .and_then(|delta| DateTime::UNIX_EPOCH.checked_add_signed(delta))
            .ok_or(NumpyDateTimeError::Unrepresentable {
                unit: Self::Days,
                val,
            })?
            .naive_utc();
        Ok(NaiveDateTime { dt, opts })
    }

    fn datetime_from_hours(val: i64, opts: Opt) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = TimeDelta::try_hours(val)
            .and_then(|delta| DateTime::UNIX_EPOCH.checked_add_signed(delta))
            .ok_or(NumpyDateTimeError::Unrepresentable {
                unit: Self::Hours,
                val,
            })?
            .naive_utc();
        Ok(NaiveDateTime { dt, opts })
    }

    fn datetime_from_minutes(val: i64, opts: Opt) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = TimeDelta::try_minutes(val)
            .and_then(|delta| DateTime::UNIX_EPOCH.checked_add_signed(delta))
            .ok_or(NumpyDateTimeError::Unrepresentable {
                unit: Self::Minutes,
                val,
            })?
            .naive_utc();
        Ok(NaiveDateTime { dt, opts })
    }

    fn datetime_from_seconds(val: i64, opts: Opt) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = TimeDelta::try_seconds(val)
            .and_then(|delta| DateTime::UNIX_EPOCH.checked_add_signed(delta))
            .ok_or(NumpyDateTimeError::Unrepresentable {
                unit: Self::Seconds,
                val,
            })?
            .naive_utc();
        Ok(NaiveDateTime { dt, opts })
    }

    fn datetime_from_milliseconds(
        val: i64,
        opts: Opt,
    ) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = TimeDelta::try_milliseconds(val)
            .and_then(|delta| DateTime::UNIX_EPOCH.checked_add_signed(delta))
            .ok_or(NumpyDateTimeError::Unrepresentable {
                unit: Self::Milliseconds,
                val,
            })?
            .naive_utc();
        Ok(NaiveDateTime { dt, opts })
    }

    fn datetime_from_microseconds(
        val: i64,
        opts: Opt,
    ) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = DateTime::UNIX_EPOCH
            .checked_add_signed(TimeDelta::microseconds(val))
            .ok_or(NumpyDateTimeError::Unrepresentable {
                unit: Self::Microseconds,
                val,
            })?
            .naive_utc();
        Ok(NaiveDateTime { dt, opts })
    }

    fn datetime_from_nanoseconds(val: i64, opts: Opt) -> Result<NaiveDateTime, NumpyDateTimeError> {
        Self::check_nat(val)?;
        let dt = DateTime::UNIX_EPOCH
            .checked_add_signed(TimeDelta::nanoseconds(val))
            .ok_or(NumpyDateTimeError::Unrepresentable {
                unit: Self::Nanoseconds,
                val,
            })?
            .naive_utc();
        Ok(NaiveDateTime { dt, opts })
    }

    fn converter(&self) -> Result<NumpyDatetimeConverter, NumpyDateTimeError> {
        match self {
            Self::Years => Ok(Self::datetime_from_years),
            Self::Months => Ok(Self::datetime_from_months),
            Self::Weeks => Ok(Self::datetime_from_weeks),
            Self::Days => Ok(Self::datetime_from_days),
            Self::Hours => Ok(Self::datetime_from_hours),
            Self::Minutes => Ok(Self::datetime_from_minutes),
            Self::Seconds => Ok(Self::datetime_from_seconds),
            Self::Milliseconds => Ok(Self::datetime_from_milliseconds),
            Self::Microseconds => Ok(Self::datetime_from_microseconds),
            Self::Nanoseconds => Ok(Self::datetime_from_nanoseconds),
            _ => Err(NumpyDateTimeError::UnsupportedUnit(*self)),
        }
    }
}

macro_rules! define_numpy_type {
    ($name:ident, $object_name:ident, $type:ty, $type_name:ident) => {
        #[repr(C)]
        struct $object_name {
            ob_base: PyObject,
            value: $type,
        }

        #[repr(transparent)]
        pub struct $name<'a, 'py> {
            obj: Borrowed<'a, 'py, PyAny>,
        }

        impl<'a, 'py> $name<'a, 'py> {
            #[inline]
            pub fn try_new(obj: BorrowedWithType<'a, 'py>, types: &NumpyTypes) -> Option<Self> {
                if obj.get_type_ptr() == types.$type_name.as_ptr().cast() {
                    Some(Self {
                        obj: obj.as_borrowed(),
                    })
                } else {
                    None
                }
            }
        }

        impl Serialize for $name<'_, '_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let value = unsafe { (*self.obj.as_ptr().cast::<$object_name>()).value };
                value.serialize(serializer)
            }
        }
    };
}

define_numpy_type!(NumpyBool, NumpyBoolObject, bool, bool_);
define_numpy_type!(NumpyFloat32, NumpyFloat32Object, f32, float32);
define_numpy_type!(NumpyFloat64, NumpyFloat64Object, f64, float64);
define_numpy_type!(NumpyInt8, NumpyInt8Object, i8, int8);
define_numpy_type!(NumpyInt16, NumpyInt16Object, i16, int16);
define_numpy_type!(NumpyInt32, NumpyInt32Object, i32, int32);
define_numpy_type!(NumpyInt64, NumpyInt64Object, i64, int64);
define_numpy_type!(NumpyUint8, NumpyUint8Object, u8, uint8);
define_numpy_type!(NumpyUint16, NumpyUint16Object, u16, uint16);
define_numpy_type!(NumpyUint32, NumpyUint32Object, u32, uint32);
define_numpy_type!(NumpyUint64, NumpyUint64Object, u64, uint64);

#[repr(C)]
struct NumpyDatetime64Object {
    ob_base: PyObject,
    value: i64,
}

pub struct NumpyDatetime64<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
    state: &'a State,
    opts: Opt,
}

impl<'a, 'py> NumpyDatetime64<'a, 'py> {
    #[inline]
    pub fn try_new(
        obj: BorrowedWithType<'a, 'py>,
        types: &NumpyTypes,
        state: &'a State,
        opts: Opt,
    ) -> Option<Self> {
        if obj.get_type_ptr() == types.datetime64.as_ptr().cast() {
            Some(Self {
                obj: obj.as_borrowed(),
                state,
                opts,
            })
        } else {
            None
        }
    }
}

impl Serialize for NumpyDatetime64<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Some(descr) = get_dtype_descr(self.obj, self.state) else {
            return Err(serde::ser::Error::custom("numpy object is malformed"));
        };
        let unit = NumpyDatetimeUnit::from_str(descr.as_borrowed());
        let value = unsafe { (*self.obj.as_ptr().cast::<NumpyDatetime64Object>()).value };
        unit.datetime(value, self.opts)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

#[repr(C)]
struct NumpyFloat16Object {
    ob_base: PyObject,
    value: u16,
}

#[repr(transparent)]
pub struct NumpyFloat16<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
}

impl<'a, 'py> NumpyFloat16<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, types: &NumpyTypes) -> Option<Self> {
        if obj.get_type_ptr() == types.float16.as_ptr().cast() {
            Some(Self {
                obj: obj.as_borrowed(),
            })
        } else {
            None
        }
    }
}

impl Serialize for NumpyFloat16<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = unsafe { (*self.obj.as_ptr().cast::<NumpyFloat16Object>()).value };
        half::f16::from_bits(value).to_f32().serialize(serializer)
    }
}
