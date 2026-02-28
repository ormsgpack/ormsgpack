// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::io::Read;
use crate::msgpack::{read_timestamp, Marker, RECURSION_LIMIT};
use simdutf8::basic::{from_utf8, Utf8Error};

#[derive(Debug)]
pub enum Error {
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

pub struct Validator<R> {
    data: R,
    recursion: u8,
}

impl<R> Validator<R>
where
    R: Read,
{
    pub fn new(data: R) -> Self {
        Validator {
            data: data,
            recursion: 0,
        }
    }

    fn validate_ext(&mut self, len: u32) -> Result<(), Error> {
        let tag = self.data.read_i8()?;
        if tag == -1 {
            read_timestamp(&mut self.data, len)?;
        } else {
            self.data.read_slice(len as usize)?;
        }
        Ok(())
    }

    fn validate_str(&mut self, len: u32) -> Result<(), Error> {
        let data = self.data.read_slice(len as usize)?;
        from_utf8(data)?;
        Ok(())
    }

    fn validate_bin(&mut self, len: u32) -> Result<(), Error> {
        self.data.read_slice(len as usize)?;
        Ok(())
    }

    fn validate_array(&mut self, len: u32) -> Result<(), Error> {
        for _ in 0..len {
            self.validate()?;
        }
        Ok(())
    }

    fn validate_map(&mut self, len: u32) -> Result<(), Error> {
        for _ in 0..len {
            self.validate()?;
            self.validate()?;
        }
        Ok(())
    }

    pub fn validate(&mut self) -> Result<(), Error> {
        self.recursion += 1;
        if unlikely!(self.recursion == RECURSION_LIMIT) {
            return Err(Error::RecursionLimitReached);
        }

        let marker = Marker::from_u8(self.data.read_u8()?);
        match marker {
            Marker::Null => Ok(()),
            Marker::True => Ok(()),
            Marker::False => Ok(()),
            Marker::FixPos(_) => Ok(()),
            Marker::U8 => {
                self.data.read_u8()?;
                Ok(())
            }
            Marker::U16 => {
                self.data.read_u16()?;
                Ok(())
            }
            Marker::U32 => {
                self.data.read_u32()?;
                Ok(())
            }
            Marker::U64 => {
                self.data.read_u64()?;
                Ok(())
            }
            Marker::FixNeg(_) => Ok(()),
            Marker::I8 => {
                self.data.read_i8()?;
                Ok(())
            }
            Marker::I16 => {
                self.data.read_i16()?;
                Ok(())
            }
            Marker::I32 => {
                self.data.read_i32()?;
                Ok(())
            }
            Marker::I64 => {
                self.data.read_i64()?;
                Ok(())
            }
            Marker::F32 => {
                self.data.read_f32()?;
                Ok(())
            }
            Marker::F64 => {
                self.data.read_f64()?;
                Ok(())
            }
            Marker::FixStr(len) => self.validate_str(len.into()),
            Marker::Str8 => {
                let len = self.data.read_u8()?;
                self.validate_str(len.into())
            }
            Marker::Str16 => {
                let len = self.data.read_u16()?;
                self.validate_str(len.into())
            }
            Marker::Str32 => {
                let len = self.data.read_u32()?;
                self.validate_str(len)
            }
            Marker::Bin8 => {
                let len = self.data.read_u8()?;
                self.validate_bin(len.into())
            }
            Marker::Bin16 => {
                let len = self.data.read_u16()?;
                self.validate_bin(len.into())
            }
            Marker::Bin32 => {
                let len = self.data.read_u32()?;
                self.validate_bin(len)
            }
            Marker::FixArray(len) => self.validate_array(len.into()),
            Marker::Array16 => {
                let len = self.data.read_u16()?;
                self.validate_array(len.into())
            }
            Marker::Array32 => {
                let len = self.data.read_u32()?;
                self.validate_array(len)
            }
            Marker::FixMap(len) => self.validate_map(len.into()),
            Marker::Map16 => {
                let len = self.data.read_u16()?;
                self.validate_map(len.into())
            }
            Marker::Map32 => {
                let len = self.data.read_u32()?;
                self.validate_map(len)
            }
            Marker::FixExt1 => self.validate_ext(1),
            Marker::FixExt2 => self.validate_ext(2),
            Marker::FixExt4 => self.validate_ext(4),
            Marker::FixExt8 => self.validate_ext(8),
            Marker::FixExt16 => self.validate_ext(16),
            Marker::Ext8 => {
                let len = self.data.read_u8()?;
                self.validate_ext(len.into())
            }
            Marker::Ext16 => {
                let len = self.data.read_u16()?;
                self.validate_ext(len.into())
            }
            Marker::Ext32 => {
                let len = self.data.read_u32()?;
                self.validate_ext(len)
            }
            Marker::Reserved => Err(Error::InvalidType(Marker::Reserved)),
        }?;

        self.recursion -= 1;
        if self.recursion == 0 && !self.data.eof() {
            Err(Error::InvalidValue)
        } else {
            Ok(())
        }
    }
}
