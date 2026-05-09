// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod neon;

pub fn count_chars(bytes: &[u8]) -> usize {
    let len = bytes.len();

    if len >= 16 {
        return unsafe { neon::count_chars(bytes) };
    }

    super::count_chars_scalar(bytes)
}
