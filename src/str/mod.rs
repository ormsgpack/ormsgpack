// SPDX-License-Identifier: (Apache-2.0 OR MIT)

macro_rules! define_count_chars {
    ($T1:ty, $T2:ty, $(#[$attrs:meta])*) => {
        $(#[$attrs])*
        pub fn count_chars(bytes: &[u8]) -> usize {
            const N1: usize = <$T1>::LEN;
            const N2: usize = <$T2>::LEN;
            let mut count = 0;
            let (chunks, remainder) = bytes.as_chunks::<N1>();

            for chunk_batch in chunks.chunks(255) {
                let mut count_vec = <$T1>::splat0();
                for chunk in chunk_batch {
                    let vec = <$T1>::from_array(chunk);
                    count_vec = count_vec.sub(&vec.mask_utf8_continuation_bytes());
                }
                count += count_vec.reduce_sum();
            }

            if !remainder.is_empty() {
                let mut count_vec = <$T2>::splat0();
                let (chunks, remainder) = remainder.as_chunks::<N2>();

                for chunk in chunks {
                    let vec = <$T2>::from_array(chunk);
                    count_vec = count_vec.sub(&vec.mask_utf8_continuation_bytes());
                }

                if !remainder.is_empty() {
                    const MASK: [u8; N2 * 2] = const {
                        let mut mask = [0u8; N2 * 2];
                        let mut i = N2;
                        while i < N2 * 2 {
                            mask[i] = 0xff;
                            i += 1;
                        }
                        mask
                    };
                    let chunk = bytes.last_chunk::<N2>().unwrap();
                    let vec = <$T2>::from_array(chunk);
                    let mask_chunk = MASK[remainder.len()..].first_chunk::<N2>().unwrap();
                    let mask_vec = <$T2>::from_array(mask_chunk);
                    count_vec = count_vec.sub(&vec.mask_utf8_continuation_bytes().and(&mask_vec));
                }

                count += count_vec.reduce_sum();
            }

            bytes.len() - count
        }
    }
}

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::count_chars;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::count_chars;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
pub use count_chars_scalar as count_chars;

#[inline]
fn is_utf8_non_continuation_byte(byte: u8) -> bool {
    (byte as i8) >= -64
}

#[inline]
fn mask_utf8_non_continuation_bytes(v: u64) -> u64 {
    ((!v >> 7) | (v >> 6)) & 0x0101010101010101
}

#[inline]
fn reduce_sum(v: u64) -> usize {
    const MASK: u64 = 0x00ff00ff00ff00ff;
    let sums = (v & MASK) + ((v >> 8) & MASK);
    (sums.wrapping_mul(0x0001000100010001) >> 48) as usize
}

#[inline]
pub fn count_chars_scalar(bytes: &[u8]) -> usize {
    let mut count = 0;
    let (chunks, remainder) = bytes.as_chunks::<8>();

    for chunk_batch in chunks.chunks(255) {
        let mut count_vec = 0;
        for chunk in chunk_batch {
            let vec = u64::from_ne_bytes(*chunk);
            count_vec += mask_utf8_non_continuation_bytes(vec);
        }
        count += reduce_sum(count_vec);
    }

    remainder
        .iter()
        .filter(|&&byte| is_utf8_non_continuation_byte(byte))
        .count()
        + count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_chars() {
        let p = "aα∞🚀";
        #[rustfmt::skip]
        let lengths = [
            8 + 2,
            16 + 4,
            32 + 8,
            64 + 6,
            64 * (255 + 1) + 32 + 4,
        ];
        for len in lengths {
            let s = p.repeat(len / p.len());
            assert_eq!(count_chars(s.as_bytes()), s.chars().count());
        }
    }
}
