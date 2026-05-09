// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[rustfmt::skip]
use std::arch::aarch64::{
    uint8x16_t,
    vaddlvq_u8,
    vandq_u8,
    vcgtq_s8,
    vdupq_n_s8,
    vdupq_n_u8,
    vld1q_u8,
    vld1q_u8_x4,
    vreinterpretq_s8_u8,
    vsubq_u8,
};

pub struct U8x16(uint8x16_t);

impl U8x16 {
    pub const LEN: usize = 16;

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn from_array(array: &[u8; 16]) -> Self {
        unsafe { Self(vld1q_u8(array.as_ptr())) }
    }

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn splat0() -> Self {
        Self(vdupq_n_u8(0))
    }

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn and(&self, other: &Self) -> Self {
        Self(vandq_u8(self.0, other.0))
    }

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        Self(vsubq_u8(self.0, other.0))
    }

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn mask_utf8_continuation_bytes(&self) -> Self {
        let v = vdupq_n_s8(-64);
        Self(vcgtq_s8(v, vreinterpretq_s8_u8(self.0)))
    }

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn reduce_sum(&self) -> usize {
        vaddlvq_u8(self.0).into()
    }
}

pub struct U8x64(uint8x16_t, uint8x16_t, uint8x16_t, uint8x16_t);

impl U8x64 {
    pub const LEN: usize = 64;

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn from_array(array: &[u8; 64]) -> Self {
        unsafe {
            let v = vld1q_u8_x4(array.as_ptr());
            Self(v.0, v.1, v.2, v.3)
        }
    }

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn splat0() -> Self {
        let v = vdupq_n_u8(0);
        Self(v, v, v, v)
    }

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        Self(
            vsubq_u8(self.0, other.0),
            vsubq_u8(self.1, other.1),
            vsubq_u8(self.2, other.2),
            vsubq_u8(self.3, other.3),
        )
    }

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn mask_utf8_continuation_bytes(&self) -> Self {
        let v = vdupq_n_s8(-64);
        Self(
            vcgtq_s8(v, vreinterpretq_s8_u8(self.0)),
            vcgtq_s8(v, vreinterpretq_s8_u8(self.1)),
            vcgtq_s8(v, vreinterpretq_s8_u8(self.2)),
            vcgtq_s8(v, vreinterpretq_s8_u8(self.3)),
        )
    }

    #[target_feature(enable = "neon")]
    #[inline]
    pub fn reduce_sum(&self) -> usize {
        (vaddlvq_u8(self.0) + vaddlvq_u8(self.1) + vaddlvq_u8(self.2) + vaddlvq_u8(self.3)).into()
    }
}

define_count_chars!(U8x64, U8x16, #[target_feature(enable = "neon")]);
