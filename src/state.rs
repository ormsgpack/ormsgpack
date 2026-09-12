// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize;
use crate::serialize;
use pyo3::prelude::*;

pub struct State {
    pub serialize: serialize::State,
    pub deserialize: deserialize::State,
}

impl State {
    #[cold]
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            serialize: serialize::State::new(py)?,
            deserialize: deserialize::State::new(py),
        })
    }
}
