// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
use crate::opt::*;

use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::PyString;

pub struct Str<'a, 'py> {
    obj: Borrowed<'a, 'py, PyString>,
    opts: Opt,
}

impl<'a, 'py> Str<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, opts: Opt) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyString>()?,
            opts: opts,
        })
    }
}

impl Serialize for Str<'_, '_> {
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        match unicode_to_str(self.obj) {
            Ok(val) => serializer.serialize_str(val),
            Err(err) => {
                if self.opts & REPLACE_SURROGATES != 0 {
                    let encoded = unicode_encode(self.obj)?;
                    let value = unsafe { std::str::from_utf8_unchecked(encoded.as_bytes()) };
                    serializer.serialize_str(value)
                } else {
                    Err(Error::custom(err))
                }
            }
        }
    }
}

pub struct StrSubclass<'a, 'py> {
    obj: Borrowed<'a, 'py, PyString>,
    opts: Opt,
}

impl<'a, 'py> StrSubclass<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, opts: Opt) -> Option<Self> {
        Some(Self {
            obj: obj.cast::<PyString>()?,
            opts: opts,
        })
    }
}

impl Serialize for StrSubclass<'_, '_> {
    #[inline(never)]
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        match unicode_to_str_via_ffi(self.obj) {
            Ok(val) => serializer.serialize_str(val),
            Err(err) => {
                if self.opts & REPLACE_SURROGATES != 0 {
                    let encoded = unicode_encode(self.obj)?;
                    let value = unsafe { std::str::from_utf8_unchecked(encoded.as_bytes()) };
                    serializer.serialize_str(value)
                } else {
                    Err(Error::custom(err))
                }
            }
        }
    }
}
