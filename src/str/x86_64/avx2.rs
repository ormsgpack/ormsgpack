// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[rustfmt::skip]
use std::arch::x86_64::{
    __m256i,
    _mm256_and_si256,
    _mm256_castsi256_si128,
    _mm256_cmpgt_epi8,
    _mm256_extracti128_si256,
    _mm256_loadu_si256,
    _mm256_sad_epu8,
    _mm256_set1_epi8,
    _mm256_setzero_si256,
    _mm256_sub_epi8,
    _mm_add_epi64,
    _mm_cvtsi128_si64,
    _mm_srli_si128,
};

#[target_feature(enable = "avx2")]
#[inline]
fn reduce_sum(a: __m256i) -> usize {
    let sums4 = _mm256_sad_epu8(a, _mm256_setzero_si256());
    let sums2 = _mm_add_epi64(
        _mm256_castsi256_si128(sums4),
        _mm256_extracti128_si256(sums4, 1),
    );
    let sum = _mm_add_epi64(sums2, _mm_srli_si128(sums2, 8));
    _mm_cvtsi128_si64(sum) as usize
}

pub struct U8x32(__m256i);

impl U8x32 {
    pub const LEN: usize = 32;

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn from_array(array: &[u8; 32]) -> Self {
        unsafe {
            let ptr = array.as_ptr().cast::<__m256i>();
            Self(_mm256_loadu_si256(ptr))
        }
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn splat0() -> Self {
        Self(_mm256_setzero_si256())
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn and(&self, other: &Self) -> Self {
        Self(_mm256_and_si256(self.0, other.0))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        Self(_mm256_sub_epi8(self.0, other.0))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn mask_utf8_continuation_bytes(&self) -> Self {
        let v = _mm256_set1_epi8(-64);
        Self(_mm256_cmpgt_epi8(v, self.0))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn reduce_sum(&self) -> usize {
        reduce_sum(self.0)
    }
}

pub struct U8x64(__m256i, __m256i);

impl U8x64 {
    pub const LEN: usize = 64;

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn from_array(array: &[u8; 64]) -> Self {
        unsafe {
            let ptr = array.as_ptr().cast::<__m256i>();
            Self(_mm256_loadu_si256(ptr), _mm256_loadu_si256(ptr.add(1)))
        }
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn splat0() -> Self {
        let v = _mm256_setzero_si256();
        Self(v, v)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        Self(
            _mm256_sub_epi8(self.0, other.0),
            _mm256_sub_epi8(self.1, other.1),
        )
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn mask_utf8_continuation_bytes(&self) -> Self {
        let v = _mm256_set1_epi8(-64);
        Self(_mm256_cmpgt_epi8(v, self.0), _mm256_cmpgt_epi8(v, self.1))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn reduce_sum(&self) -> usize {
        reduce_sum(self.0) + reduce_sum(self.1)
    }
}

define_count_chars!(U8x64, U8x32, #[target_feature(enable = "avx2")]);
