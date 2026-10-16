// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::gc::{PyTraverseError, PyVisit};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyInt};

#[pyclass(name = "Ext", module = "ormsgpack", frozen)]
pub struct PyExt {
    pub tag: Py<PyInt>,
    pub data: Py<PyBytes>,
}

#[pymethods]
impl PyExt {
    #[new]
    #[pyo3(signature = (tag, data, /))]
    fn new(tag: Py<PyInt>, data: Py<PyBytes>) -> Self {
        Self { tag, data }
    }

    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.tag)?;
        visit.call(&self.data)
    }
}
