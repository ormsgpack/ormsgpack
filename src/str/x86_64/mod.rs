// SPDX-License-Identifier: (Apache-2.0 OR MIT)

mod avx2;
mod sse2;

pub fn count_chars(bytes: &[u8]) -> usize {
    let len = bytes.len();

    if len >= 32 && is_x86_feature_detected!("avx2") {
        return unsafe { avx2::count_chars(bytes) };
    }

    if len >= 16 {
        return unsafe { sse2::count_chars(bytes) };
    }

    super::count_chars_scalar(bytes)
}
