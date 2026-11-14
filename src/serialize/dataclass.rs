// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::ffi::*;
use crate::util::unlikely;

use crate::io::WriteSlices;
use crate::serialize::Error;
use crate::serialize::{Serialize, Serializer};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use smallvec::SmallVec;

pub struct State {
    field_type: Py<PyAny>,
    dict_str: Py<PyString>,
    slots_str: Py<PyString>,
    dataclass_fields_str: Py<PyString>,
    field_type_str: Py<PyString>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            dict_str: PyString::intern(py, "__dict__").unbind(),
            slots_str: PyString::intern(py, "__slots__").unbind(),
            field_type: py.import("dataclasses")?.getattr("_FIELD")?.unbind(),
            dataclass_fields_str: PyString::intern(py, "__dataclass_fields__").unbind(),
            field_type_str: PyString::intern(py, "_field_type").unbind(),
        })
    }
}

pub struct Dataclass<'a, 'py> {
    obj: Borrowed<'a, 'py, PyAny>,
    state: &'a State,
}

impl<'a, 'py> Dataclass<'a, 'py> {
    #[inline]
    pub fn try_new(obj: BorrowedWithType<'a, 'py>, state: &'a State) -> Option<Self> {
        let dataclass_fields_str = state.dataclass_fields_str.bind_borrowed(obj.py());
        if get_type_dict(obj.get_type()).is_some_and(|v| v.contains(dataclass_fields_str).unwrap())
        {
            Some(Self {
                obj: obj.as_borrowed(),
                state: state,
            })
        } else {
            None
        }
    }

    fn get_field_value(
        &self,
        name: Borrowed<'_, 'py, PyString>,
        field: Borrowed<'_, 'py, PyAny>,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        let state = self.state;
        let field_type_str = state.field_type_str.bind_borrowed(field.py());
        let field_type = field.getattr(field_type_str)?;
        if field_type.as_ptr() == state.field_type.as_ptr() {
            let value = self.obj.getattr(name)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

impl Serialize for Dataclass<'_, '_> {
    fn serialize_as<const KEY: bool, W>(
        &self,
        serializer: &mut Serializer<'_, '_, W>,
    ) -> Result<(), Error>
    where
        W: WriteSlices,
    {
        let state = self.state;
        let Some(fields) = self
            .obj
            .getattr(state.dataclass_fields_str.bind_borrowed(self.obj.py()))
            .map(cast_into_exact::<PyDict>)?
        else {
            return Err(Error::custom("__dataclass_fields__ must be a dict"));
        };
        let len = fields.len();
        if unlikely(len == 0) {
            return serializer.serialize_map(0, |_| Ok(()));
        }

        let maybe_dict = {
            let slots_str = state.slots_str.bind_borrowed(self.obj.py());
            let type_obj = get_type(self.obj);
            if get_type_dict(type_obj).is_some_and(|v| v.contains(slots_str).unwrap()) {
                None
            } else {
                let dict_str = state.dict_str.bind_borrowed(self.obj.py());
                let Some(dict) = self.obj.getattr(dict_str).map(cast_into_exact::<PyDict>)? else {
                    return Err(Error::custom("__dict__ attribute must be a dict"));
                };
                Some(dict)
            }
        };

        let mut items: SmallVec<[(&str, Bound<'_, PyAny>); 8]> = SmallVec::with_capacity(len);
        for (key, field) in PyDictIter::new(fields.as_borrowed()) {
            let Some(key) = cast_exact::<PyString>(key) else {
                return Err(Error::custom(KEY_MUST_BE_STR));
            };
            let key_as_str = unicode_to_str(key).map_err(Error::custom)?;
            if key_as_str.as_bytes()[0] == b'_' {
                continue;
            }

            if let Some(dict) = &maybe_dict {
                if let Some(value) = dict.get_item(key)? {
                    items.push((key_as_str, value));
                } else if let Some(value) = self.get_field_value(key, field)? {
                    items.push((key_as_str, value));
                }
            } else {
                if let Some(value) = self.get_field_value(key, field)? {
                    items.push((key_as_str, value));
                }
            }
        }

        serializer.serialize_map(items.len(), |map| {
            for (key, value) in items.iter() {
                map.serialize_entry(key, &value.as_borrowed())?
            }
            Ok(())
        })
    }
}
