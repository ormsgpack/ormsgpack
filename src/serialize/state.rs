// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use super::{dataclass, datetime, enum_, ext, fragment, numpy, pydantic, uuid};

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};
use pyo3::PyTypeInfo;

#[allow(non_snake_case)]
pub struct State {
    pub ext: ext::State,
    pub fragment: fragment::State,
    pub dataclass: dataclass::State,
    pub datetime: datetime::State,
    pub enum_: enum_::State,
    pub numpy: numpy::State,
    pub pydantic: pydantic::State,
    pub uuid: uuid::State,
    pub dict_str: Py<PyString>,
    pub slots_str: Py<PyString>,
    pub MsgpackEncodeError: Py<PyType>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            ext: ext::State::new(py)?,
            fragment: fragment::State::new(py)?,
            dataclass: dataclass::State::new(py)?,
            datetime: datetime::State::new(py),
            enum_: enum_::State::new(py)?,
            numpy: numpy::State::new(py),
            pydantic: pydantic::State::new(py),
            uuid: uuid::State::new(py)?,
            dict_str: PyString::intern(py, "__dict__").unbind(),
            slots_str: PyString::intern(py, "__slots__").unbind(),
            MsgpackEncodeError: PyTypeError::type_object(py).unbind(),
        })
    }

    #[cold]
    pub fn error(&self, py: Python<'_>, message: &str) -> PyErr {
        let message = PyString::new(py, message).unbind();
        PyErr::from_type(
            self.MsgpackEncodeError.bind_borrowed(py).to_owned(),
            message,
        )
    }
}
