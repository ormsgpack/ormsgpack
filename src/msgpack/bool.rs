// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::msgpack::marker::Marker;

#[inline]
pub fn write_bool<W>(writer: &mut W, value: bool) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    if value {
        writer.write_all(&[Marker::True.into()])
    } else {
        writer.write_all(&[Marker::False.into()])
    }
}
