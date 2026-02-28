// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
use crate::opt::*;
use crate::serialize::datetimelike::{DateLike, DateTimeLike, TimeLike};
use crate::state::State;
use serde::ser::{Serialize, Serializer};
use serde_bytes::Bytes;

#[repr(transparent)]
pub struct Date {
    ptr: *mut pyo3::ffi::PyObject,
}

impl Date {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        Date { ptr: ptr }
    }
}

impl DateLike for Date {
    fn year(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_GET_YEAR(self.ptr) as i32 }
    }

    fn month(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_GET_MONTH(self.ptr) as i32 }
    }

    fn day(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_GET_DAY(self.ptr) as i32 }
    }
}

impl Serialize for Date {
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

pub struct Time {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
}

impl Time {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, opts: Opt) -> Result<Self, TimeError> {
        let tzinfo = unsafe { pyo3::ffi::PyDateTime_TIME_GET_TZINFO(ptr) };
        if tzinfo != unsafe { pyo3::ffi::Py_None() } {
            return Err(TimeError::HasTimezone);
        }
        Ok(Time {
            ptr: ptr,
            opts: opts,
        })
    }
}

impl TimeLike for Time {
    fn hour(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_TIME_GET_HOUR(self.ptr) as i32 }
    }

    fn minute(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_TIME_GET_MINUTE(self.ptr) as i32 }
    }

    fn second(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_TIME_GET_SECOND(self.ptr) as i32 }
    }

    fn microsecond(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_TIME_GET_MICROSECOND(self.ptr) as i32 }
    }
}

impl Serialize for Time {
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

unsafe fn utcoffset(
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
) -> Result<Option<i32>, DateTimeError> {
    let tzinfo = pyo3::ffi::PyDateTime_DATE_GET_TZINFO(ptr);
    if tzinfo == unsafe { pyo3::ffi::Py_None() } {
        return Ok(None);
    }
    let py_offset: *mut pyo3::ffi::PyObject;
    if pyo3::ffi::PyObject_HasAttr(tzinfo, (*state).normalize_str) == 1 {
        // pytz
        let normalized = pyobject_call_method_one_arg(tzinfo, (*state).normalize_str, ptr);
        py_offset = pyobject_call_method_no_args(normalized, (*state).utcoffset_str);
        pyo3::ffi::Py_DECREF(normalized);
    } else {
        py_offset = pyobject_call_method_one_arg(tzinfo, (*state).utcoffset_str, ptr);
    }
    if unlikely!(py_offset.is_null()) {
        pyo3::ffi::PyErr_Clear();
        return Err(DateTimeError::LibraryUnsupported);
    }
    let day = pyo3::ffi::PyDateTime_DELTA_GET_DAYS(py_offset);
    let second = pyo3::ffi::PyDateTime_DELTA_GET_SECONDS(py_offset);
    pyo3::ffi::Py_DECREF(py_offset);
    let offset = if day == -1 {
        // datetime.timedelta(days=-1, seconds=68400) -> -05:00
        -86400 + second
    } else {
        // datetime.timedelta(seconds=37800) -> +10:30
        second
    };
    Ok(Some(offset))
}

pub struct DateTime {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    offset: Option<i32>,
}

impl DateTime {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        state: *mut State,
        opts: Opt,
    ) -> Result<Self, DateTimeError> {
        let offset = unsafe { utcoffset(ptr, state)? };
        Ok(DateTime {
            ptr: ptr,
            opts: opts,
            offset: offset,
        })
    }
}

impl DateLike for DateTime {
    fn year(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_GET_YEAR(self.ptr) as i32 }
    }

    fn month(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_GET_MONTH(self.ptr) as i32 }
    }

    fn day(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_GET_DAY(self.ptr) as i32 }
    }
}

impl TimeLike for DateTime {
    fn hour(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_DATE_GET_HOUR(self.ptr) as i32 }
    }

    fn minute(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_DATE_GET_MINUTE(self.ptr) as i32 }
    }

    fn second(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_DATE_GET_SECOND(self.ptr) as i32 }
    }

    fn microsecond(&self) -> i32 {
        unsafe { pyo3::ffi::PyDateTime_DATE_GET_MICROSECOND(self.ptr) as i32 }
    }
}

impl DateTimeLike for DateTime {
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

impl Serialize for DateTime {
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
