// SPDX-License-Identifier: (Apache-2.0 OR MIT)
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::ptr_eq)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::zero_prefixed_literal)]
#![deny(clippy::ptr_as_ptr)]

#[macro_use]
mod util;

mod deserialize;
mod exc;
mod ext;
mod ffi;
mod fragment;
mod io;
mod msgpack;
mod opt;
mod serialize;
mod state;
mod str;

use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyBytes, PyInt};

static STATE: PyOnceLock<state::State> = PyOnceLock::new();

#[pymodule(gil_used = false)]
fn ormsgpack(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    let state = STATE.get_or_try_init(py, || state::State::new(py))?;

    module.add_function(wrap_pyfunction!(packb, module)?)?;
    module.add_function(wrap_pyfunction!(unpackb, module)?)?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add("Ext", state.serialize.ext.type_object.bind(py))?;
    module.add("Fragment", state.serialize.fragment.type_object.bind(py))?;
    module.add(
        "MsgpackDecodeError",
        state.deserialize.MsgpackDecodeError.bind(py),
    )?;
    module.add(
        "MsgpackEncodeError",
        state.serialize.MsgpackEncodeError.bind(py),
    )?;

    module.add(
        "OPT_DATETIME_AS_TIMESTAMP_EXT",
        opt::DATETIME_AS_TIMESTAMP_EXT,
    )?;
    module.add("OPT_NAIVE_UTC", opt::NAIVE_UTC)?;
    module.add("OPT_NON_STR_KEYS", opt::NON_STR_KEYS)?;
    module.add("OPT_OMIT_MICROSECONDS", opt::OMIT_MICROSECONDS)?;
    module.add("OPT_PASSTHROUGH_BIG_INT", opt::PASSTHROUGH_BIG_INT)?;
    module.add("OPT_PASSTHROUGH_DATACLASS", opt::PASSTHROUGH_DATACLASS)?;
    module.add("OPT_PASSTHROUGH_DATETIME", opt::PASSTHROUGH_DATETIME)?;
    module.add("OPT_PASSTHROUGH_ENUM", opt::PASSTHROUGH_ENUM)?;
    module.add("OPT_PASSTHROUGH_SUBCLASS", opt::PASSTHROUGH_SUBCLASS)?;
    module.add("OPT_PASSTHROUGH_TUPLE", opt::PASSTHROUGH_TUPLE)?;
    module.add("OPT_PASSTHROUGH_UUID", opt::PASSTHROUGH_UUID)?;
    module.add("OPT_REPLACE_SURROGATES", opt::REPLACE_SURROGATES)?;
    module.add("OPT_SERIALIZE_NUMPY", opt::SERIALIZE_NUMPY)?;
    module.add("OPT_SERIALIZE_PYDANTIC", opt::SERIALIZE_PYDANTIC)?;
    module.add("OPT_SORT_KEYS", opt::SORT_KEYS)?;
    module.add("OPT_UTC_Z", opt::UTC_Z)?;
    Ok(())
}

fn parse_option_arg(opts: Option<&Bound<'_, PyAny>>, mask: opt::Opt) -> Result<opt::Opt, ()> {
    let Some(opts) = opts else {
        return Ok(0);
    };
    if opts.is_exact_instance_of::<PyInt>() {
        let val = opts.extract::<opt::Opt>().map_err(|_| ())?;
        if val & !mask == 0 {
            Ok(val)
        } else {
            Err(())
        }
    } else if opts.is_none() {
        Ok(0)
    } else {
        Err(())
    }
}

/// Serialize Python objects to msgpack.
#[pyfunction(signature = (obj, /, default = None, option = None))]
fn packb<'py>(
    obj: &Bound<'py, PyAny>,
    default: Option<&Bound<'py, PyAny>>,
    option: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyBytes>> {
    let state = STATE.get(obj.py()).unwrap();
    let opts = parse_option_arg(option, opt::PACKB_OPT_MASK)
        .map_err(|()| state.serialize.error(obj.py(), "Invalid opts"))?;
    serialize::serialize(
        obj.as_borrowed(),
        &state.serialize,
        default.map(Bound::as_borrowed),
        opts,
    )
}

/// Deserialize msgpack to Python objects.
#[pyfunction(signature = (obj, /, *, ext_hook = None, option = None))]
fn unpackb<'py>(
    obj: &Bound<'py, PyAny>,
    ext_hook: Option<&Bound<'py, PyAny>>,
    option: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyAny>> {
    let state = STATE.get(obj.py()).unwrap();
    let opts = parse_option_arg(option, opt::UNPACKB_OPT_MASK)
        .map_err(|()| state.deserialize.error(obj.py(), "Invalid opts"))?;
    deserialize::deserialize(
        obj.as_borrowed(),
        &state.deserialize,
        ext_hook.map(Bound::as_borrowed),
        opts,
    )
}
