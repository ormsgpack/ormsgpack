// SPDX-License-Identifier: (Apache-2.0 OR MIT)

macro_rules! ob_type {
    ($obj:expr) => {
        unsafe { (*$obj.cast::<pyo3::ffi::PyObject>()).ob_type }
    };
}

#[cfg(feature = "nightly")]
macro_rules! unlikely {
    ($exp:expr) => {
        core::intrinsics::unlikely($exp)
    };
}

#[cfg(not(feature = "nightly"))]
macro_rules! unlikely {
    ($exp:expr) => {
        $exp
    };
}
