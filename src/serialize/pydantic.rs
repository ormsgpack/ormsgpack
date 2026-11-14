// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::ffi::*;
use crate::opt::*;
use crate::util::unlikely;

use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use smallvec::SmallVec;

pub struct State {
    fields_str: Py<PyString>,
    dict_str: Py<PyString>,
    pydantic_extra_str: Py<PyString>,
    pydantic_validator_str: Py<PyString>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> Self {
        Self {
            dict_str: PyString::intern(py, "__dict__").unbind(),
            fields_str: PyString::intern(py, "__fields__").unbind(),
            pydantic_extra_str: PyString::intern(py, "__pydantic_extra__").unbind(),
            pydantic_validator_str: PyString::intern(py, "__pydantic_validator__").unbind(),
        }
    }
}

pub struct PydanticModel<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
    state: &'a State,
    opts: Opt,
}

impl<'a, 'py> PydanticModel<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, state: &'a State, opts: Opt) -> Option<Self> {
        if get_type_dict(obj.get_type()).is_some_and(|v| {
            let fields_str = state.fields_str.bind_borrowed(obj.py());
            let pydantic_validator_str = state.pydantic_validator_str.bind_borrowed(obj.py());
            v.contains(fields_str).unwrap() || v.contains(pydantic_validator_str).unwrap()
        }) {
            Some(Self {
                obj: obj.as_borrowed(),
                state: state,
                opts: opts,
            })
        } else {
            None
        }
    }
}

impl Serialize for PydanticModel<'_, '_> {
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let state = self.state;
        let Some(dict) = self
            .obj
            .getattr(state.dict_str.bind_borrowed(self.obj.py()))
            .map(cast_into_exact::<PyDict>)?
        else {
            return Err(Error::custom("__dict__ attribute must be a dict"));
        };

        if let Some(extra_dict) = self
            .obj
            .getattr_opt(state.pydantic_extra_str.bind_borrowed(self.obj.py()))?
            .and_then(cast_into_exact::<PyDict>)
        {
            self.serialize_with_extra(serializer, dict.as_borrowed(), extra_dict.as_borrowed())
        } else {
            self.serialize_with_no_extra(serializer, dict.as_borrowed())
        }
    }
}

impl<'a, 'py> PydanticModel<'a, 'py> {
    fn serialize_with_no_extra<W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
        dict: Borrowed<'a, 'py, PyDict>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let len = dict.len();
        if unlikely(len == 0) {
            return serializer.serialize_map(0, |_| Ok(()));
        }
        let mut items: SmallVec<[(&str, Borrowed<'a, 'py, PyAny>); 8]> =
            SmallVec::with_capacity(len);
        for (key, value) in PyDictIter::new(dict) {
            let Some(key) = cast_exact::<PyString>(key) else {
                return Err(Error::custom(KEY_MUST_BE_STR));
            };
            let key_as_str = unicode_to_str(key).map_err(Error::custom)?;
            if unlikely(key_as_str.as_bytes()[0] == b'_') {
                continue;
            }
            items.push((key_as_str, value));
        }

        if self.opts & SORT_KEYS != 0 {
            items.sort_unstable_by(|a, b| a.0.cmp(b.0));
        }

        serializer.serialize_map(items.len(), |map| {
            for (key, value) in items.iter() {
                map.serialize_entry(key, &value)?;
            }
            Ok(())
        })
    }

    fn serialize_with_extra<W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
        dict: Borrowed<'a, 'py, PyDict>,
        extra_dict: Borrowed<'a, 'py, PyDict>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let iter = PyDictIter::new(dict).chain(PyDictIter::new(extra_dict));
        let len = iter.size_hint().0;
        if unlikely(len == 0) {
            return serializer.serialize_map(0, |_| Ok(()));
        }
        let mut items: SmallVec<[(&str, Borrowed<'a, 'py, PyAny>); 8]> =
            SmallVec::with_capacity(len);
        for (key, value) in iter {
            let Some(key) = cast_exact::<PyString>(key) else {
                return Err(Error::custom(KEY_MUST_BE_STR));
            };
            let key_as_str = unicode_to_str(key).map_err(Error::custom)?;
            if unlikely(key_as_str.as_bytes()[0] == b'_') {
                continue;
            }
            items.push((key_as_str, value));
        }

        if self.opts & SORT_KEYS != 0 {
            items.sort_unstable_by(|a, b| a.0.cmp(b.0));
        }

        serializer.serialize_map(items.len(), |map| {
            for (key, value) in items.iter() {
                map.serialize_entry(key, &value)?;
            }
            Ok(())
        })
    }
}
