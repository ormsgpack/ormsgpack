// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::ffi::*;
use crate::io::WriteSlices;
use crate::opt::*;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use crate::util::unlikely;
use pyo3::prelude::*;
use pyo3::types::{PyFrozenDict, PyString};
use smallvec::SmallVec;

pub struct FrozenDict<'a, 'py> {
    obj: Borrowed<'a, 'py, PyFrozenDict>,
    opts: Opt,
}

impl<'a, 'py> FrozenDict<'a, 'py> {
    #[inline]
    pub fn try_new_exact(obj: BorrowedWithType<'a, 'py>, opts: Opt) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyFrozenDict>()?,
            opts: opts,
        })
    }

    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, opts: Opt) -> Option<Self> {
        Some(Self {
            obj: obj.cast::<PyFrozenDict>()?,
            opts: opts,
        })
    }
}

impl Serialize for FrozenDict<'_, '_> {
    #[inline]
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        if unlikely(self.obj.len() == 0) {
            serializer.serialize_map(0, |_| Ok(()))
        } else if self.opts & (NON_STR_KEYS | SORT_KEYS) == 0 {
            self.serialize_with_str_keys(serializer)
        } else if self.opts & NON_STR_KEYS != 0 {
            if self.opts & SORT_KEYS != 0 {
                return Err(Error::custom(
                    "OPT_NON_STR_KEYS is not compatible with OPT_SORT_KEYS",
                ));
            }
            self.serialize_with_non_str_keys(serializer)
        } else {
            self.serialize_with_sorted_str_keys(serializer)
        }
    }
}

impl<'a, 'py> FrozenDict<'a, 'py> {
    #[inline(always)]
    fn serialize_with_str_keys<W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let len = self.obj.len();
        serializer.serialize_map(len, |map| {
            for (key, value) in PyFrozenDictIter::new(self.obj) {
                let Some(key) = cast_exact::<PyString>(key) else {
                    return Err(Error::custom(KEY_MUST_BE_STR));
                };
                let key_as_str = unicode_to_str(key).map_err(Error::custom)?;
                map.serialize_entry(key_as_str, &value)?;
            }
            Ok(())
        })
    }

    #[inline(never)]
    fn serialize_with_sorted_str_keys<W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let len = self.obj.len();
        let mut items: SmallVec<[(&str, Borrowed<'a, 'py, PyAny>); 8]> =
            SmallVec::with_capacity(len);
        for (key, value) in PyFrozenDictIter::new(self.obj) {
            let Some(key) = cast_exact::<PyString>(key) else {
                return Err(Error::custom(KEY_MUST_BE_STR));
            };
            let key_as_str = unicode_to_str(key).map_err(Error::custom)?;
            items.push((key_as_str, value));
        }

        items.sort_unstable_by(|a, b| a.0.cmp(b.0));

        serializer.serialize_map(len, |map| {
            for (key, val) in items.iter() {
                map.serialize_entry(key, &val)?;
            }
            Ok(())
        })
    }

    #[inline(never)]
    fn serialize_with_non_str_keys<W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let len = self.obj.len();
        serializer.serialize_map(len, |map| {
            for (key, value) in PyFrozenDictIter::new(self.obj) {
                if let Some(key) = cast_exact::<PyString>(key) {
                    let key_as_str = unicode_to_str(key).map_err(Error::custom)?;
                    map.serialize_entry(key_as_str, &value)?;
                } else {
                    map.serialize_entry(&key, &value)?;
                }
            }
            Ok(())
        })
    }
}
