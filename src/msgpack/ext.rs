// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::io::{Read, WriteSlices};
use crate::msgpack::marker::Marker;

pub fn write_ext<W>(writer: &mut W, value: &[u8], tag: i8) -> Result<(), std::io::Error>
where
    W: WriteSlices,
{
    let len = value.len();
    if len == 1 {
        writer.write_slices([&[Marker::FixExt1.into(), tag as u8], value])
    } else if len == 2 {
        writer.write_slices([&[Marker::FixExt2.into(), tag as u8], value])
    } else if len == 4 {
        writer.write_slices([&[Marker::FixExt4.into(), tag as u8], value])
    } else if len == 8 {
        writer.write_slices([&[Marker::FixExt8.into(), tag as u8], value])
    } else if len == 16 {
        writer.write_slices([&[Marker::FixExt16.into(), tag as u8], value])
    } else if len < 256 {
        writer.write_slices([&[Marker::Ext8.into(), len as u8, tag as u8], value])
    } else if len < 65536 {
        writer.write_slices([
            &[Marker::Ext16.into()],
            &(len as u16).to_be_bytes(),
            &[tag as u8],
            value,
        ])
    } else if len <= 4294967295 {
        writer.write_slices([
            &[Marker::Ext32.into()],
            &(len as u32).to_be_bytes(),
            &[tag as u8],
            value,
        ])
    } else {
        Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
    }
}

pub fn read_timestamp<R>(
    reader: &mut R,
    len: u32,
) -> Result<chrono::DateTime<chrono::Utc>, std::io::Error>
where
    R: Read,
{
    let (seconds, nanoseconds): (i64, u32) = match len {
        4 => {
            let seconds = reader.read_u32()?;
            (seconds.into(), 0)
        }
        8 => {
            let value = reader.read_u64()?;
            ((value & 0x3ffffffff) as i64, (value >> 34) as u32)
        }
        12 => {
            let nanoseconds = reader.read_u32()?;
            let seconds = reader.read_i64()?;
            (seconds, nanoseconds)
        }
        _ => return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)),
    };
    match chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, nanoseconds) {
        Some(value) => Ok(value),
        None => Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)),
    }
}

pub fn write_timestamp<W>(
    writer: &mut W,
    datetime: chrono::DateTime<chrono::Utc>,
) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    let seconds = datetime.timestamp();
    let nanoseconds = datetime.timestamp_subsec_nanos();
    if seconds >> 34 == 0 {
        let value = (i64::from(nanoseconds) << 34) | seconds;
        if value <= 4294967295 {
            writer.write_all(&(value as u32).to_be_bytes())?;
        } else {
            writer.write_all(&(value as u64).to_be_bytes())?;
        }
    } else {
        writer.write_all(&nanoseconds.to_be_bytes())?;
        writer.write_all(&seconds.to_be_bytes())?;
    }
    Ok(())
}
