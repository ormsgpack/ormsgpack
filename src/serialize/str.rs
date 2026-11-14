// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::*;
use crate::opt::*;
use crate::util::unlikely;

use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};

#[repr(transparent)]
struct StrWithSurrogates<'a, 'py> {
    obj: Borrowed<'a, 'py, PyString>,
}

impl<'a, 'py> StrWithSurrogates<'a, 'py> {
    pub fn new(obj: Borrowed<'a, 'py, PyString>) -> Self {
        Self { obj: obj }
    }
}

impl Serialize for StrWithSurrogates<'_, '_> {
    #[inline(never)]
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        unsafe {
            let ptr = pyo3::ffi::PyUnicode_AsEncodedString(
                self.obj.as_ptr(),
                c"UTF-8".as_ptr(),
                c"replace".as_ptr(),
            );
            if unlikely(ptr.is_null()) {
                return Err(Error::custom("invalid string"));
            }
            let obj = Bound::from_owned_ptr(self.obj.py(), ptr).cast_into_unchecked::<PyBytes>();
            let uni = std::str::from_utf8_unchecked(obj.as_bytes());
            serializer.serialize_str(uni)
        }
    }
}

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
                    StrWithSurrogates::new(self.obj).serialize_as::<KEY, _>(serializer)
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
                    StrWithSurrogates::new(self.obj).serialize_as::<KEY, _>(serializer)
                } else {
                    Err(Error::custom(err))
                }
            }
        }
    }
}
