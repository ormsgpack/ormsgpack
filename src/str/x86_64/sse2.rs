// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[rustfmt::skip]
use std::arch::x86_64::{
    __m128i,
    _mm_add_epi64,
    _mm_and_si128,
    _mm_cmpgt_epi8,
    _mm_cvtsi128_si64,
    _mm_loadu_si128,
    _mm_sad_epu8,
    _mm_set1_epi8,
    _mm_setzero_si128,
    _mm_srli_si128,
    _mm_sub_epi8,
};

#[target_feature(enable = "sse2")]
#[inline]
fn reduce_sum(a: __m128i) -> usize {
    let sums = _mm_sad_epu8(a, _mm_setzero_si128());
    let sum = _mm_add_epi64(sums, _mm_srli_si128(sums, 8));
    _mm_cvtsi128_si64(sum) as usize
}

pub struct U8x16(__m128i);

impl U8x16 {
    pub const LEN: usize = 16;

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn from_array(array: &[u8; 16]) -> Self {
        unsafe {
            let ptr = array.as_ptr().cast::<__m128i>();
            Self(_mm_loadu_si128(ptr))
        }
    }

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn splat0() -> Self {
        Self(_mm_setzero_si128())
    }

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn and(&self, other: &Self) -> Self {
        Self(_mm_and_si128(self.0, other.0))
    }

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        Self(_mm_sub_epi8(self.0, other.0))
    }

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn mask_utf8_continuation_bytes(&self) -> Self {
        let v = _mm_set1_epi8(-64);
        Self(_mm_cmpgt_epi8(v, self.0))
    }

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn reduce_sum(&self) -> usize {
        reduce_sum(self.0)
    }
}

pub struct U8x64(__m128i, __m128i, __m128i, __m128i);

impl U8x64 {
    pub const LEN: usize = 64;

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn from_array(array: &[u8; 64]) -> Self {
        unsafe {
            let ptr = array.as_ptr().cast::<__m128i>();
            Self(
                _mm_loadu_si128(ptr),
                _mm_loadu_si128(ptr.add(1)),
                _mm_loadu_si128(ptr.add(2)),
                _mm_loadu_si128(ptr.add(3)),
            )
        }
    }

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn splat0() -> Self {
        let v = _mm_setzero_si128();
        Self(v, v, v, v)
    }

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        Self(
            _mm_sub_epi8(self.0, other.0),
            _mm_sub_epi8(self.1, other.1),
            _mm_sub_epi8(self.2, other.2),
            _mm_sub_epi8(self.3, other.3),
        )
    }

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn mask_utf8_continuation_bytes(&self) -> Self {
        let v = _mm_set1_epi8(-64);
        Self(
            _mm_cmpgt_epi8(v, self.0),
            _mm_cmpgt_epi8(v, self.1),
            _mm_cmpgt_epi8(v, self.2),
            _mm_cmpgt_epi8(v, self.3),
        )
    }

    #[target_feature(enable = "sse2")]
    #[inline]
    pub fn reduce_sum(&self) -> usize {
        reduce_sum(self.0) + reduce_sum(self.1) + reduce_sum(self.2) + reduce_sum(self.3)
    }
}

define_count_chars!(U8x64, U8x16, #[target_feature(enable = "sse2")]);
