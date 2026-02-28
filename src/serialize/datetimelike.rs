// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::msgpack;
use crate::opt::*;
use chrono::{Datelike, Timelike};
use serde::ser::{Serialize, Serializer};

fn write_integer<W>(writer: &mut W, value: i32, width: usize) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    let mut itoa_buf = itoa::Buffer::new();
    let formatted = itoa_buf.format(value);
    for _ in 0..width - formatted.len() {
        writer.write_all(b"0")?;
    }
    let len = writer.write(formatted.as_bytes())?;
    debug_assert!(len == formatted.len());
    Ok(())
}

pub trait DateLike {
    fn year(&self) -> i32;
    fn month(&self) -> i32;
    fn day(&self) -> i32;

    fn write_rfc3339<W>(&self, writer: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        write_integer(writer, self.year(), 4)?;
        writer.write_all(b"-")?;
        write_integer(writer, self.month(), 2)?;
        writer.write_all(b"-")?;
        write_integer(writer, self.day(), 2)?;
        Ok(())
    }
}

pub trait TimeLike {
    fn hour(&self) -> i32;
    fn minute(&self) -> i32;
    fn second(&self) -> i32;
    fn microsecond(&self) -> i32;

    fn write_rfc3339<W>(&self, writer: &mut W, opts: Opt) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        write_integer(writer, self.hour(), 2)?;
        writer.write_all(b":")?;
        write_integer(writer, self.minute(), 2)?;
        writer.write_all(b":")?;
        write_integer(writer, self.second(), 2)?;
        if opts & OMIT_MICROSECONDS == 0 {
            let microsecond = self.microsecond();
            if microsecond != 0 {
                writer.write_all(b".")?;
                write_integer(writer, microsecond, 6)?;
            }
        }
        Ok(())
    }
}

pub trait DateTimeLike: DateLike + TimeLike {
    fn offset(&self) -> Option<i32>;
    fn to_utc_datetime(&self) -> chrono::DateTime<chrono::Utc>;

    fn write_rfc3339<W>(&self, writer: &mut W, opts: Opt) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        DateLike::write_rfc3339(self, writer)?;
        writer.write_all(b"T")?;
        TimeLike::write_rfc3339(self, writer, opts)?;
        if self.offset().is_some() || opts & NAIVE_UTC != 0 {
            let offset = self.offset().unwrap_or_default();
            if offset == 0 {
                if opts & UTC_Z != 0 {
                    writer.write_all(b"Z")?;
                } else {
                    let tz = b"+00:00";
                    let len = writer.write(tz)?;
                    debug_assert!(len == tz.len());
                }
            } else {
                let offset_hour: i32;
                let mut offset_minute: i32;
                let mut offset_second: i32;
                if offset < 0 {
                    writer.write_all(b"-")?;
                    offset_second = -offset;
                } else {
                    writer.write_all(b"+")?;
                    offset_second = offset;
                }
                (offset_minute, offset_second) = (offset_second / 60, offset_second % 60);
                (offset_hour, offset_minute) = (offset_minute / 60, offset_minute % 60);
                // https://tools.ietf.org/html/rfc3339#section-5.8
                // "exactly 19 minutes and 32.13 seconds ahead of UTC"
                // "closest representable UTC offset"
                //  "+20:00"
                if offset_second >= 30 {
                    offset_minute += 1;
                }
                write_integer(writer, offset_hour, 2)?;
                writer.write_all(b":")?;
                write_integer(writer, offset_minute, 2)?;
            }
        }
        Ok(())
    }

    fn write_timestamp<W>(&self, writer: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        msgpack::write_timestamp(writer, self.to_utc_datetime())
    }
}

pub struct NaiveDateTime {
    pub dt: chrono::NaiveDateTime,
    pub opts: Opt,
}

impl DateLike for NaiveDateTime {
    fn year(&self) -> i32 {
        self.dt.year()
    }

    fn month(&self) -> i32 {
        self.dt.month() as i32
    }

    fn day(&self) -> i32 {
        self.dt.day() as i32
    }
}

impl TimeLike for NaiveDateTime {
    fn hour(&self) -> i32 {
        self.dt.hour() as i32
    }

    fn minute(&self) -> i32 {
        self.dt.minute() as i32
    }

    fn second(&self) -> i32 {
        self.dt.second() as i32
    }

    fn microsecond(&self) -> i32 {
        (self.dt.nanosecond() / 1_000) as i32
    }
}

impl DateTimeLike for NaiveDateTime {
    fn offset(&self) -> Option<i32> {
        None
    }

    fn to_utc_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        self.dt.and_utc()
    }
}

impl Serialize for NaiveDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut cursor = std::io::Cursor::new([0u8; 32]);
        DateTimeLike::write_rfc3339(self, &mut cursor, self.opts).unwrap();
        let len = cursor.position() as usize;
        let value = unsafe { std::str::from_utf8_unchecked(&cursor.get_ref()[0..len]) };
        serializer.serialize_str(value)
    }
}
