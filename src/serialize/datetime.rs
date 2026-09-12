// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::{cast_into_exact, hasattr, BorrowedWithType};
use crate::opt::*;
use crate::serialize::datetimelike::{DateLike, DateTimeLike, TimeLike};
use pyo3::prelude::*;
use pyo3::types::{
    PyDate, PyDateAccess, PyDateTime, PyDelta, PyDeltaAccess, PyString, PyTime, PyTimeAccess,
    PyType, PyTzInfoAccess,
};
use pyo3::PyTypeInfo;
use serde::ser::{Serialize, Serializer};
use serde_bytes::Bytes;

pub struct State {
    date_type: Py<PyType>,
    time_type: Py<PyType>,
    datetime_type: Py<PyType>,
    normalize_str: Py<PyString>,
    utcoffset_str: Py<PyString>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> Self {
        Self {
            date_type: PyDate::type_object(py).unbind(),
            time_type: PyTime::type_object(py).unbind(),
            datetime_type: PyDateTime::type_object(py).unbind(),
            normalize_str: PyString::intern(py, "normalize").unbind(),
            utcoffset_str: PyString::intern(py, "utcoffset").unbind(),
        }
    }
}

#[repr(transparent)]
pub struct Date<'a, 'py> {
    obj: Borrowed<'a, 'py, PyDate>,
}

impl<'a, 'py> Date<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, state: &State) -> Option<Self> {
        if obj.get_type_ptr() == state.date_type.as_ptr().cast() {
            Some(Self {
                obj: unsafe { obj.as_borrowed().cast_unchecked() },
            })
        } else {
            None
        }
    }
}

impl DateLike for Date<'_, '_> {
    fn year(&self) -> i32 {
        self.obj.get_year()
    }

    fn month(&self) -> i32 {
        self.obj.get_month().into()
    }

    fn day(&self) -> i32 {
        self.obj.get_day().into()
    }
}

impl Serialize for Date<'_, '_> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut cursor = std::io::Cursor::new([0u8; 32]);
        DateLike::write_rfc3339(self, &mut cursor).unwrap();
        let len = cursor.position() as usize;
        let value = unsafe { std::str::from_utf8_unchecked(&cursor.get_ref()[0..len]) };
        serializer.serialize_str(value)
    }
}

pub enum TimeError {
    HasTimezone,
}

impl std::fmt::Display for TimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HasTimezone => write!(f, "datetime.time must not have tzinfo set"),
        }
    }
}

pub struct Time<'a, 'py> {
    obj: Borrowed<'a, 'py, PyTime>,
    opts: Opt,
}

impl<'a, 'py> Time<'a, 'py> {
    #[inline]
    pub fn try_new(
        obj: BorrowedWithType<'a, 'py>,
        state: &State,
        opts: Opt,
    ) -> Result<Option<Self>, TimeError> {
        if obj.get_type_ptr() != state.time_type.as_ptr().cast() {
            return Ok(None);
        }
        let obj = unsafe { obj.as_borrowed().cast_unchecked::<PyTime>() };
        if obj.get_tzinfo().is_some() {
            return Err(TimeError::HasTimezone);
        }
        Ok(Some(Self {
            obj: obj,
            opts: opts,
        }))
    }
}

impl TimeLike for Time<'_, '_> {
    fn hour(&self) -> i32 {
        self.obj.get_hour().into()
    }

    fn minute(&self) -> i32 {
        self.obj.get_minute().into()
    }

    fn second(&self) -> i32 {
        self.obj.get_second().into()
    }

    fn microsecond(&self) -> i32 {
        self.obj.get_microsecond() as i32
    }
}

impl Serialize for Time<'_, '_> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut cursor = std::io::Cursor::new([0u8; 32]);
        TimeLike::write_rfc3339(self, &mut cursor, self.opts).unwrap();
        let len = cursor.position() as usize;
        let value = unsafe { std::str::from_utf8_unchecked(&cursor.get_ref()[0..len]) };
        serializer.serialize_str(value)
    }
}

pub enum DateTimeError {
    LibraryUnsupported,
}

impl std::fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibraryUnsupported => write!(f, "datetime's timezone library is not supported: use datetime.timezone.utc, pendulum, pytz, or dateutil"),
        }
    }
}

fn utcoffset(
    obj: &Borrowed<'_, '_, PyDateTime>,
    state: &State,
) -> Result<Option<i32>, DateTimeError> {
    let Some(tzinfo) = obj.get_tzinfo() else {
        return Ok(None);
    };
    let delta = {
        let normalize_str = state.normalize_str.bind_borrowed(obj.py());
        let utcoffset_str = state.utcoffset_str.bind_borrowed(obj.py());
        hasattr(tzinfo.as_borrowed(), normalize_str).and_then(|has_normalize| {
            if has_normalize {
                tzinfo
                    .call_method1(normalize_str, (obj,))
                    .and_then(|normalized| normalized.call_method0(utcoffset_str))
            } else {
                tzinfo.call_method1(utcoffset_str, (obj,))
            }
        })
    }
    .ok()
    .and_then(cast_into_exact::<PyDelta>)
    .ok_or(DateTimeError::LibraryUnsupported)?;
    let day = delta.get_days();
    let second = delta.get_seconds();
    let offset = if day == -1 {
        // datetime.timedelta(days=-1, seconds=68400) -> -05:00
        -86400 + second
    } else {
        // datetime.timedelta(seconds=37800) -> +10:30
        second
    };
    Ok(Some(offset))
}

pub struct DateTime<'a, 'py> {
    obj: Borrowed<'a, 'py, PyDateTime>,
    opts: Opt,
    offset: Option<i32>,
}

impl<'a, 'py> DateTime<'a, 'py> {
    #[inline(always)]
    pub fn try_new(
        obj: BorrowedWithType<'a, 'py>,
        state: &State,
        opts: Opt,
    ) -> Result<Option<Self>, DateTimeError> {
        if obj.get_type_ptr() != state.datetime_type.as_ptr().cast() {
            return Ok(None);
        }
        let obj = unsafe { obj.as_borrowed().cast_unchecked::<PyDateTime>() };
        let offset = utcoffset(&obj, state)?;
        Ok(Some(Self {
            obj: obj,
            offset: offset,
            opts: opts,
        }))
    }
}

impl DateLike for DateTime<'_, '_> {
    fn year(&self) -> i32 {
        self.obj.get_year()
    }

    fn month(&self) -> i32 {
        self.obj.get_month().into()
    }

    fn day(&self) -> i32 {
        self.obj.get_day().into()
    }
}

impl TimeLike for DateTime<'_, '_> {
    fn hour(&self) -> i32 {
        self.obj.get_hour().into()
    }

    fn minute(&self) -> i32 {
        self.obj.get_minute().into()
    }

    fn second(&self) -> i32 {
        self.obj.get_second().into()
    }

    fn microsecond(&self) -> i32 {
        self.obj.get_microsecond() as i32
    }
}

impl DateTimeLike for DateTime<'_, '_> {
    fn offset(&self) -> Option<i32> {
        self.offset
    }

    fn to_utc_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        let offset = chrono::FixedOffset::east_opt(self.offset.unwrap_or_default()).unwrap();
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(self.year(), self.month() as u32, self.day() as u32)
                .unwrap(),
            chrono::NaiveTime::from_hms_micro_opt(
                self.hour() as u32,
                self.minute() as u32,
                self.second() as u32,
                self.microsecond() as u32,
            )
            .unwrap(),
        )
        .and_local_timezone(offset)
        .unwrap()
        .into()
    }
}

impl Serialize for DateTime<'_, '_> {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut cursor = std::io::Cursor::new([0u8; 32]);
        if self.opts & DATETIME_AS_TIMESTAMP_EXT != 0
            && (self.offset().is_some() || self.opts & NAIVE_UTC != 0)
        {
            DateTimeLike::write_timestamp(self, &mut cursor).unwrap();
            let len = cursor.position() as usize;
            let timestamp = &cursor.get_ref()[0..len];
            serializer.serialize_newtype_variant("", 128, "", Bytes::new(timestamp))
        } else {
            DateTimeLike::write_rfc3339(self, &mut cursor, self.opts).unwrap();
            let len = cursor.position() as usize;
            let value = unsafe { std::str::from_utf8_unchecked(&cursor.get_ref()[0..len]) };
            serializer.serialize_str(value)
        }
    }
}
