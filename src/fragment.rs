// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::msgpack;
use pyo3::exceptions::PyValueError;
use pyo3::gc::{PyTraverseError, PyVisit};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyclass(name = "Fragment", module = "ormsgpack", frozen)]
pub struct PyFragment {
    pub data: Py<PyBytes>,
}

#[pymethods]
impl PyFragment {
    #[new]
    #[pyo3(signature = (data, /))]
    fn new(py: Python<'_>, data: Py<PyBytes>) -> PyResult<Self> {
        let contents = data.bind(py).as_bytes();
        let mut validator = msgpack::Validator::new(contents);
        validator
            .validate()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(Self { data })
    }

    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.data)
    }
}
