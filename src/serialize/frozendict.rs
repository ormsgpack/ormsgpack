// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::ffi::*;
use crate::opt::*;
use crate::serialize::serializer::*;
use crate::serialize::{Context, DictKeyContext};
use crate::util::unlikely;
use pyo3::prelude::*;
use pyo3::types::{PyFrozenDict, PyString};
use serde::ser::{Serialize, SerializeMap, Serializer};
use smallvec::SmallVec;

pub struct FrozenDict<'a, 'py> {
    obj: Borrowed<'a, 'py, PyFrozenDict>,
    context: Context<'a, 'py>,
}

impl<'a, 'py> FrozenDict<'a, 'py> {
    #[inline]
    pub fn try_new_exact(
        obj: BorrowedWithType<'a, 'py>,
        context: Context<'a, 'py>,
    ) -> Option<Self> {
        Some(Self {
            obj: obj.cast_exact::<PyFrozenDict>()?,
            context: context,
        })
    }

    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, context: Context<'a, 'py>) -> Option<Self> {
        Some(Self {
            obj: obj.cast::<PyFrozenDict>()?,
            context: context,
        })
    }
}

impl Serialize for FrozenDict<'_, '_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if unlikely(self.obj.len() == 0) {
            serializer.serialize_map(Some(0))?.end()
        } else if self.context.opts & (NON_STR_KEYS | SORT_KEYS) == 0 {
            self.serialize_with_str_keys(serializer)
        } else if self.context.opts & NON_STR_KEYS != 0 {
            if self.context.opts & SORT_KEYS != 0 {
                return Err(serde::ser::Error::custom(
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
    fn serialize_with_str_keys<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = self.obj.len();
        let mut map = serializer.serialize_map(Some(len))?;
        for (key, value) in PyFrozenDictIter::new(self.obj) {
            let Some(key) = cast_exact::<PyString>(key) else {
                return Err(serde::ser::Error::custom(KEY_MUST_BE_STR));
            };
            let key_as_str = unicode_to_str(key).map_err(serde::ser::Error::custom)?;
            let pyvalue = PyObject::new(value, self.context);
            map.serialize_key(key_as_str).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }

    #[inline(never)]
    fn serialize_with_sorted_str_keys<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = self.obj.len();
        let mut items: SmallVec<[(&str, Borrowed<'a, 'py, PyAny>); 8]> =
            SmallVec::with_capacity(len);
        for (key, value) in PyFrozenDictIter::new(self.obj) {
            let Some(key) = cast_exact::<PyString>(key) else {
                return Err(serde::ser::Error::custom(KEY_MUST_BE_STR));
            };
            let key_as_str = unicode_to_str(key).map_err(serde::ser::Error::custom)?;
            items.push((key_as_str, value));
        }

        items.sort_unstable_by(|a, b| a.0.cmp(b.0));

        let mut map = serializer.serialize_map(Some(len))?;
        for (key, val) in items.iter() {
            let pyvalue = PyObject::new(*val, self.context);
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }

    #[inline(never)]
    fn serialize_with_non_str_keys<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = self.obj.len();
        let mut map = serializer.serialize_map(Some(len))?;
        for (key, value) in PyFrozenDictIter::new(self.obj) {
            if let Some(key) = cast_exact::<PyString>(key) {
                let key_as_str = unicode_to_str(key).map_err(serde::ser::Error::custom)?;
                map.serialize_entry(key_as_str, &PyObject::new(value, self.context))?;
            } else {
                map.serialize_entry(
                    &DictKey::new(
                        key,
                        DictKeyContext {
                            state: self.context.state,
                            opts: self.context.opts,
                        },
                    ),
                    &PyObject::new(value, self.context),
                )?;
            }
        }
        map.end()
    }
}
