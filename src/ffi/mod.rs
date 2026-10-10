// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod buffer;
mod dict;
#[cfg(Py_3_15)]
mod frozendict;
#[cfg_attr(any(PyPy, GraalPy), path = "base/mod.rs")]
#[cfg_attr(not(any(PyPy, GraalPy)), path = "cpython/mod.rs")]
mod impl_;
mod int;
mod object;
mod unicode;

pub use buffer::*;
pub use dict::*;
#[cfg(Py_3_15)]
pub use frozendict::*;
pub use impl_::*;
pub use int::*;
pub use object::*;
pub use unicode::*;
