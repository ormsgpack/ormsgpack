// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use super::cache::KeyMap;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};
use pyo3::PyTypeInfo;

#[allow(non_snake_case)]
pub struct State {
    pub MsgpackDecodeError: Py<PyType>,
    pub key_map: KeyMap<512>,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> Self {
        Self {
            MsgpackDecodeError: PyValueError::type_object(py).unbind(),
            key_map: KeyMap::new(),
        }
    }

    #[cold]
    pub fn error(&self, py: Python<'_>, message: &str) -> PyErr {
        let message = PyString::new(py, message).unbind();
        PyErr::from_type(
            self.MsgpackDecodeError.bind_borrowed(py).to_owned(),
            message,
        )
    }
}
