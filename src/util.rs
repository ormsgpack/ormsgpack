// SPDX-License-Identifier: (Apache-2.0 OR MIT)

macro_rules! ob_type {
    ($obj:expr) => {
        unsafe { (*$obj.cast::<pyo3::ffi::PyObject>()).ob_type }
    };
}

// TODO: replace with core::hint::cold_path once 1.95.0 is old enough
#[inline(always)]
#[cold]
pub const fn cold_path() {}

#[inline(always)]
pub const fn unlikely(b: bool) -> bool {
    if b {
        cold_path();
    }
    b
}
