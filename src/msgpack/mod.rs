// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod array;
mod bin;
mod bool;
mod ext;
mod float;
mod int;
mod map;
mod marker;
mod nil;
mod serializer;
mod str;
mod validator;

pub use array::*;
pub use bin::*;
pub use bool::*;
pub use ext::*;
pub use float::*;
pub use int::*;
pub use map::*;
pub use marker::*;
pub use nil::*;
pub use serializer::*;
pub use str::*;
pub use validator::*;

pub const RECURSION_LIMIT: u8 = 255;
