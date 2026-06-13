// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::state::State;
use serde::ser::{Serialize, Serializer};
use std::os::raw::c_uchar;

pub struct UUID {
    ptr: *mut pyo3::ffi::PyObject,
    state: *mut State,
}

const HEX: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
];

fn write_group<W>(writer: &mut W, group: &[c_uchar]) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    for i in 0..group.len() {
        writer.write_all(&[
            HEX[(group[i] >> 4) as usize],
            HEX[(group[i] & 0x0f) as usize],
        ])?;
    }
    Ok(())
}

impl UUID {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, state: *mut State) -> Self {
        UUID {
            ptr: ptr,
            state: state,
        }
    }
    pub fn write_buf<W>(&self, writer: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        let mut buffer: [c_uchar; 16] = [0; 16];
        unsafe {
            let value = pyo3::ffi::PyObject_GetAttr(self.ptr, (*self.state).int_str);
            #[cfg(Py_3_13)]
            {
                pyo3::ffi::PyLong_AsNativeBytes(
                    value,
                    buffer.as_mut_ptr().cast(),
                    16,
                    pyo3::ffi::Py_ASNATIVEBYTES_BIG_ENDIAN
                        | pyo3::ffi::Py_ASNATIVEBYTES_UNSIGNED_BUFFER
                        | pyo3::ffi::Py_ASNATIVEBYTES_REJECT_NEGATIVE,
                );
            }
            #[cfg(not(Py_3_13))]
            {
                pyo3::ffi::_PyLong_AsByteArray(
                    value.cast::<pyo3::ffi::PyLongObject>(),
                    buffer.as_mut_ptr(),
                    16,
                    0, // little_endian
                    0, // is_signed
                );
            }
            pyo3::ffi::Py_DECREF(value);
        };

        write_group(writer, &buffer[..4])?;
        writer.write_all(b"-")?;
        write_group(writer, &buffer[4..6])?;
        writer.write_all(b"-")?;
        write_group(writer, &buffer[6..8])?;
        writer.write_all(b"-")?;
        write_group(writer, &buffer[8..10])?;
        writer.write_all(b"-")?;
        write_group(writer, &buffer[10..])?;
        Ok(())
    }
}

impl Serialize for UUID {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut cursor = std::io::Cursor::new([0u8; 64]);
        self.write_buf(&mut cursor).unwrap();
        let len = cursor.position() as usize;
        let value = unsafe { std::str::from_utf8_unchecked(&cursor.get_ref()[0..len]) };
        serializer.serialize_str(value)
    }
}
