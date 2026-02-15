use crate::exc::*;
use crate::io::WriteSlices;
use crate::msgpack;
use serde::ser;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    Write,
}

impl std::fmt::Display for Error {
    #[cold]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Custom(ref msg) => f.write_str(msg),
            Error::Write => f.write_str("write error"),
        }
    }
}

impl From<std::io::Error> for Error {
    #[cold]
    fn from(_: std::io::Error) -> Error {
        Error::Write
    }
}

impl serde::ser::Error for Error {
    #[cold]
    fn custom<T>(msg: T) -> Error
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl std::error::Error for Error {}

struct ExtSerializer<'a, W> {
    tag: i8,
    writer: &'a mut W,
}

impl<'a, W> ExtSerializer<'a, W>
where
    W: WriteSlices,
{
    #[inline]
    fn new(tag: i8, writer: &'a mut W) -> Self {
        Self {
            tag: tag,
            writer: writer,
        }
    }
}

impl<W> ser::Serializer for &mut ExtSerializer<'_, W>
where
    W: WriteSlices,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ser::Impossible<(), Error>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        msgpack::write_ext(self.writer, value, self.tag)?;
        Ok(())
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_i8(self, _value: i8) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_i16(self, _value: i16) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_i32(self, _value: i32) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_i64(self, _value: i64) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_u8(self, _value: u8) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_u16(self, _value: u16) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_u32(self, _value: u32) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_u64(self, _value: u64) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_char(self, _value: char) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_str(self, _value: &str) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!();
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!();
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!();
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unreachable!();
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        unreachable!();
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        unreachable!();
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        unreachable!();
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        unreachable!();
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        unreachable!();
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        unreachable!();
    }
}

pub struct Serializer<W> {
    writer: W,
    recursion: u8,
}

impl<W> Serializer<W>
where
    W: WriteSlices,
{
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer {
            writer,
            recursion: 0,
        }
    }
}

pub struct Compound<'a, W> {
    se: &'a mut Serializer<W>,
}

impl<W> ser::SerializeSeq for Compound<'_, W>
where
    W: WriteSlices,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.se.recursion -= 1;
        Ok(())
    }
}

impl<W> ser::SerializeMap for Compound<'_, W>
where
    W: WriteSlices,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        key.serialize(&mut *self.se)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.se.recursion -= 1;
        Ok(())
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: WriteSlices,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        msgpack::write_bool(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        msgpack::write_i64(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&value.to_be_bytes())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(value))
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(value))
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(value))
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        msgpack::write_u64(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&value.to_be_bytes())
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        msgpack::write_f32(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        msgpack::write_f64(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_char(self, _value: char) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        msgpack::write_str(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        msgpack::write_bin(&mut self.writer, value)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        msgpack::write_nil(&mut self.writer)?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_unit_variant(
        self,
        _name: &str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unreachable!();
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!();
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let tag: i8 = match variant_index {
            128 => -1,
            _ => variant_index.try_into().unwrap_or_else(|_| unreachable!()),
        };
        let mut ext_se = ExtSerializer::new(tag, &mut self.writer);
        value.serialize(&mut ext_se)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        match len {
            Some(len) => {
                if unlikely!(self.recursion == msgpack::RECURSION_LIMIT) {
                    return Err(Error::Custom(RECURSION_LIMIT_REACHED.to_string()));
                }

                self.recursion += 1;
                msgpack::write_array_len(&mut self.writer, len)?;
                Ok(Compound { se: self })
            }
            None => unreachable!(),
        }
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unreachable!();
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unreachable!();
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        unreachable!();
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        match len {
            Some(len) => {
                if unlikely!(self.recursion == msgpack::RECURSION_LIMIT) {
                    return Err(Error::Custom(RECURSION_LIMIT_REACHED.to_string()));
                }

                self.recursion += 1;
                msgpack::write_map_len(&mut self.writer, len)?;
                Ok(Compound { se: self })
            }
            None => unreachable!(),
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unreachable!();
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        unreachable!();
    }
}
