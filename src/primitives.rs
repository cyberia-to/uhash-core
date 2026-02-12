//! Raw compression functions for UniversalHash
//!
//! These implement the spec's AES_Compress, SHA256_Compress, and BLAKE3_Compress
//! using low-level operations for maximum performance.

use crate::params::BLOCK_SIZE;

/// AES expansion: 4 AESENC rounds with a single key (for scratchpad init)
/// Input: 128-bit state, 128-bit key
/// Output: 128-bit state after 4 AESENC rounds
#[inline(always)]
pub fn aes_expand_block(state: &[u8; 16], key: &[u8; 16]) -> [u8; 16] {
    #[cfg(all(target_arch = "x86_64", target_feature = "aes"))]
    {
        aes_expand_x86(state, key)
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "aes"))]
    {
        aes_expand_arm(state, key)
    }

    #[cfg(not(any(
        all(target_arch = "x86_64", target_feature = "aes"),
        all(target_arch = "aarch64", target_feature = "aes")
    )))]
    {
        aes_expand_soft(state, key)
    }
}

/// x86_64 AES expansion
#[cfg(all(target_arch = "x86_64", target_feature = "aes"))]
#[inline(always)]
fn aes_expand_x86(state: &[u8; 16], key: &[u8; 16]) -> [u8; 16] {
    use core::arch::x86_64::{__m128i, _mm_aesenc_si128, _mm_loadu_si128, _mm_storeu_si128};

    unsafe {
        let mut s = _mm_loadu_si128(state.as_ptr() as *const __m128i);
        let k = _mm_loadu_si128(key.as_ptr() as *const __m128i);

        // 4 AESENC rounds with same key
        s = _mm_aesenc_si128(s, k);
        s = _mm_aesenc_si128(s, k);
        s = _mm_aesenc_si128(s, k);
        s = _mm_aesenc_si128(s, k);

        let mut result = [0u8; 16];
        _mm_storeu_si128(result.as_mut_ptr() as *mut __m128i, s);
        result
    }
}

/// ARM AES expansion
#[cfg(all(target_arch = "aarch64", target_feature = "aes"))]
#[inline(always)]
fn aes_expand_arm(state: &[u8; 16], key: &[u8; 16]) -> [u8; 16] {
    use core::arch::aarch64::{vaeseq_u8, vaesmcq_u8, vld1q_u8, vst1q_u8};

    unsafe {
        let mut s = vld1q_u8(state.as_ptr());
        let k = vld1q_u8(key.as_ptr());

        // 4 AESENC rounds (AESE + AESMC)
        s = vaesmcq_u8(vaeseq_u8(s, k));
        s = vaesmcq_u8(vaeseq_u8(s, k));
        s = vaesmcq_u8(vaeseq_u8(s, k));
        s = vaesmcq_u8(vaeseq_u8(s, k));

        let mut result = [0u8; 16];
        vst1q_u8(result.as_mut_ptr(), s);
        result
    }
}

/// Software AES expansion (for WASM and targets without hardware AES)
#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "aes"),
    all(target_arch = "aarch64", target_feature = "aes")
)))]
#[inline(always)]
fn aes_expand_soft(state: &[u8; 16], key: &[u8; 16]) -> [u8; 16] {
    let mut s = *state;
    // 4 AESENC rounds
    s = aesenc_round(&s, key);
    s = aesenc_round(&s, key);
    s = aesenc_round(&s, key);
    s = aesenc_round(&s, key);
    s
}

/// AES-based compression: 4 rounds of AESENC
///
/// Spec: state = AES_Compress(state, block) using 4 AESENC rounds
/// Input: 256-bit state, 512-bit block (we use first 256 bits as round keys)
#[inline(always)]
pub fn aes_compress(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
    #[cfg(all(target_arch = "x86_64", target_feature = "aes"))]
    {
        aes_compress_x86(state, block)
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "aes"))]
    {
        aes_compress_arm(state, block)
    }

    #[cfg(not(any(
        all(target_arch = "x86_64", target_feature = "aes"),
        all(target_arch = "aarch64", target_feature = "aes")
    )))]
    {
        aes_compress_soft(state, block)
    }
}

/// x86_64 AES-NI implementation
#[cfg(all(target_arch = "x86_64", target_feature = "aes"))]
#[inline(always)]
fn aes_compress_x86(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
    use core::arch::x86_64::{__m128i, _mm_aesenc_si128, _mm_loadu_si128, _mm_storeu_si128};

    unsafe {
        // Load state halves
        let mut state_lo = _mm_loadu_si128(state.as_ptr() as *const __m128i);
        let mut state_hi = _mm_loadu_si128(state.as_ptr().add(16) as *const __m128i);

        // Load round keys from block
        let key0 = _mm_loadu_si128(block.as_ptr() as *const __m128i);
        let key1 = _mm_loadu_si128(block.as_ptr().add(16) as *const __m128i);
        let key2 = _mm_loadu_si128(block.as_ptr().add(32) as *const __m128i);
        let key3 = _mm_loadu_si128(block.as_ptr().add(48) as *const __m128i);

        // 4 rounds of AESENC on low half
        state_lo = _mm_aesenc_si128(state_lo, key0);
        state_lo = _mm_aesenc_si128(state_lo, key1);
        state_lo = _mm_aesenc_si128(state_lo, key2);
        state_lo = _mm_aesenc_si128(state_lo, key3);

        // 4 rounds of AESENC on high half (using rotated keys per spec)
        state_hi = _mm_aesenc_si128(state_hi, key2);
        state_hi = _mm_aesenc_si128(state_hi, key3);
        state_hi = _mm_aesenc_si128(state_hi, key0);
        state_hi = _mm_aesenc_si128(state_hi, key1);

        // Store result
        let mut result = [0u8; 32];
        _mm_storeu_si128(result.as_mut_ptr() as *mut __m128i, state_lo);
        _mm_storeu_si128(result.as_mut_ptr().add(16) as *mut __m128i, state_hi);
        result
    }
}

/// ARM NEON + Crypto implementation
#[cfg(all(target_arch = "aarch64", target_feature = "aes"))]
#[inline(always)]
fn aes_compress_arm(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
    use core::arch::aarch64::{vaeseq_u8, vaesmcq_u8, vld1q_u8, vst1q_u8};

    unsafe {
        // Load state halves
        let mut state_lo = vld1q_u8(state.as_ptr());
        let mut state_hi = vld1q_u8(state.as_ptr().add(16));

        // Load round keys from block
        let key0 = vld1q_u8(block.as_ptr());
        let key1 = vld1q_u8(block.as_ptr().add(16));
        let key2 = vld1q_u8(block.as_ptr().add(32));
        let key3 = vld1q_u8(block.as_ptr().add(48));

        // ARM AES: vaeseq does SubBytes+ShiftRows, vaesmcq does MixColumns
        // We need to XOR the key, then apply AESE+AESMC

        // 4 rounds on low half
        state_lo = vaesmcq_u8(vaeseq_u8(state_lo, key0));
        state_lo = vaesmcq_u8(vaeseq_u8(state_lo, key1));
        state_lo = vaesmcq_u8(vaeseq_u8(state_lo, key2));
        state_lo = vaesmcq_u8(vaeseq_u8(state_lo, key3));

        // 4 rounds on high half (rotated keys)
        state_hi = vaesmcq_u8(vaeseq_u8(state_hi, key2));
        state_hi = vaesmcq_u8(vaeseq_u8(state_hi, key3));
        state_hi = vaesmcq_u8(vaeseq_u8(state_hi, key0));
        state_hi = vaesmcq_u8(vaeseq_u8(state_hi, key1));

        // Store result
        let mut result = [0u8; 32];
        vst1q_u8(result.as_mut_ptr(), state_lo);
        vst1q_u8(result.as_mut_ptr().add(16), state_hi);
        result
    }
}

/// Software fallback for AES compression (WASM, older CPUs)
/// Implements actual AESENC rounds: SubBytes + ShiftRows + MixColumns + AddRoundKey
#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "aes"),
    all(target_arch = "aarch64", target_feature = "aes")
)))]
#[inline(always)]
fn aes_compress_soft(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
    let mut result = [0u8; 32];

    // Process low half with 4 AESENC rounds using keys 0,1,2,3
    let mut state_lo: [u8; 16] = state[0..16].try_into().unwrap();
    state_lo = aesenc_round(&state_lo, &block[0..16]);
    state_lo = aesenc_round(&state_lo, &block[16..32]);
    state_lo = aesenc_round(&state_lo, &block[32..48]);
    state_lo = aesenc_round(&state_lo, &block[48..64]);

    // Process high half with rotated keys 2,3,0,1
    let mut state_hi: [u8; 16] = state[16..32].try_into().unwrap();
    state_hi = aesenc_round(&state_hi, &block[32..48]);
    state_hi = aesenc_round(&state_hi, &block[48..64]);
    state_hi = aesenc_round(&state_hi, &block[0..16]);
    state_hi = aesenc_round(&state_hi, &block[16..32]);

    result[0..16].copy_from_slice(&state_lo);
    result[16..32].copy_from_slice(&state_hi);

    result
}

/// Single AESENC round: SubBytes + ShiftRows + MixColumns + AddRoundKey
#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "aes"),
    all(target_arch = "aarch64", target_feature = "aes")
)))]
#[inline(always)]
fn aesenc_round(state: &[u8; 16], round_key: &[u8]) -> [u8; 16] {
    // SubBytes
    let mut s = [0u8; 16];
    for i in 0..16 {
        s[i] = SBOX[state[i] as usize];
    }

    // ShiftRows (in-place on s, viewed as 4x4 column-major matrix)
    // Row 0: no shift
    // Row 1: shift left by 1
    // Row 2: shift left by 2
    // Row 3: shift left by 3
    let t = s;
    s[1] = t[5];
    s[5] = t[9];
    s[9] = t[13];
    s[13] = t[1];

    s[2] = t[10];
    s[6] = t[14];
    s[10] = t[2];
    s[14] = t[6];

    s[3] = t[15];
    s[7] = t[3];
    s[11] = t[7];
    s[15] = t[11];

    // MixColumns
    let mut out = [0u8; 16];
    for col in 0..4 {
        let i = col * 4;
        let a0 = s[i];
        let a1 = s[i + 1];
        let a2 = s[i + 2];
        let a3 = s[i + 3];

        out[i] = gf_mul2(a0) ^ gf_mul3(a1) ^ a2 ^ a3;
        out[i + 1] = a0 ^ gf_mul2(a1) ^ gf_mul3(a2) ^ a3;
        out[i + 2] = a0 ^ a1 ^ gf_mul2(a2) ^ gf_mul3(a3);
        out[i + 3] = gf_mul3(a0) ^ a1 ^ a2 ^ gf_mul2(a3);
    }

    // AddRoundKey
    for i in 0..16 {
        out[i] ^= round_key[i];
    }

    out
}

/// Multiply by 2 in GF(2^8) with reduction polynomial x^8 + x^4 + x^3 + x + 1
#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "aes"),
    all(target_arch = "aarch64", target_feature = "aes")
)))]
#[inline(always)]
fn gf_mul2(x: u8) -> u8 {
    let hi = x >> 7;
    let shifted = x << 1;
    shifted ^ (hi * 0x1b)
}

/// Multiply by 3 in GF(2^8): 3*x = 2*x + x
#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "aes"),
    all(target_arch = "aarch64", target_feature = "aes")
)))]
#[inline(always)]
fn gf_mul3(x: u8) -> u8 {
    gf_mul2(x) ^ x
}

/// AES S-box (for software fallback only)
#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "aes"),
    all(target_arch = "aarch64", target_feature = "aes")
)))]
const SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

/// SHA-256 compression function
///
/// Uses the raw compression function, not the full hash
#[inline(always)]
pub fn sha256_compress(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
    #[cfg(all(target_arch = "aarch64", target_feature = "sha2"))]
    {
        sha256_compress_arm(state, block)
    }

    #[cfg(not(all(target_arch = "aarch64", target_feature = "sha2")))]
    {
        sha256_compress_soft(state, block)
    }
}

/// ARM SHA256 compression using hardware intrinsics
#[cfg(all(target_arch = "aarch64", target_feature = "sha2"))]
#[inline(always)]
fn sha256_compress_arm(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
    use core::arch::aarch64::*;

    // SHA256 round constants
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    unsafe {
        // Load state (big-endian)
        let mut state0 = vreinterpretq_u32_u8(vrev32q_u8(vld1q_u8(state.as_ptr())));
        let mut state1 = vreinterpretq_u32_u8(vrev32q_u8(vld1q_u8(state.as_ptr().add(16))));

        // Save original state for final addition
        let state0_save = state0;
        let state1_save = state1;

        // Load message block (big-endian)
        let mut msg0 = vreinterpretq_u32_u8(vrev32q_u8(vld1q_u8(block.as_ptr())));
        let mut msg1 = vreinterpretq_u32_u8(vrev32q_u8(vld1q_u8(block.as_ptr().add(16))));
        let mut msg2 = vreinterpretq_u32_u8(vrev32q_u8(vld1q_u8(block.as_ptr().add(32))));
        let mut msg3 = vreinterpretq_u32_u8(vrev32q_u8(vld1q_u8(block.as_ptr().add(48))));

        // Rounds 0-3
        let mut tmp = vaddq_u32(msg0, vld1q_u32(K.as_ptr()));
        state1 = vsha256hq_u32(state1, state0, tmp);
        state0 = vsha256h2q_u32(state0, state1_save, tmp);
        msg0 = vsha256su1q_u32(vsha256su0q_u32(msg0, msg1), msg2, msg3);

        // Rounds 4-7
        tmp = vaddq_u32(msg1, vld1q_u32(K.as_ptr().add(4)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg1 = vsha256su1q_u32(vsha256su0q_u32(msg1, msg2), msg3, msg0);

        // Rounds 8-11
        tmp = vaddq_u32(msg2, vld1q_u32(K.as_ptr().add(8)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg2 = vsha256su1q_u32(vsha256su0q_u32(msg2, msg3), msg0, msg1);

        // Rounds 12-15
        tmp = vaddq_u32(msg3, vld1q_u32(K.as_ptr().add(12)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg3 = vsha256su1q_u32(vsha256su0q_u32(msg3, msg0), msg1, msg2);

        // Rounds 16-19
        tmp = vaddq_u32(msg0, vld1q_u32(K.as_ptr().add(16)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg0 = vsha256su1q_u32(vsha256su0q_u32(msg0, msg1), msg2, msg3);

        // Rounds 20-23
        tmp = vaddq_u32(msg1, vld1q_u32(K.as_ptr().add(20)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg1 = vsha256su1q_u32(vsha256su0q_u32(msg1, msg2), msg3, msg0);

        // Rounds 24-27
        tmp = vaddq_u32(msg2, vld1q_u32(K.as_ptr().add(24)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg2 = vsha256su1q_u32(vsha256su0q_u32(msg2, msg3), msg0, msg1);

        // Rounds 28-31
        tmp = vaddq_u32(msg3, vld1q_u32(K.as_ptr().add(28)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg3 = vsha256su1q_u32(vsha256su0q_u32(msg3, msg0), msg1, msg2);

        // Rounds 32-35
        tmp = vaddq_u32(msg0, vld1q_u32(K.as_ptr().add(32)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg0 = vsha256su1q_u32(vsha256su0q_u32(msg0, msg1), msg2, msg3);

        // Rounds 36-39
        tmp = vaddq_u32(msg1, vld1q_u32(K.as_ptr().add(36)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg1 = vsha256su1q_u32(vsha256su0q_u32(msg1, msg2), msg3, msg0);

        // Rounds 40-43
        tmp = vaddq_u32(msg2, vld1q_u32(K.as_ptr().add(40)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg2 = vsha256su1q_u32(vsha256su0q_u32(msg2, msg3), msg0, msg1);

        // Rounds 44-47
        tmp = vaddq_u32(msg3, vld1q_u32(K.as_ptr().add(44)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);
        msg3 = vsha256su1q_u32(vsha256su0q_u32(msg3, msg0), msg1, msg2);

        // Rounds 48-51
        tmp = vaddq_u32(msg0, vld1q_u32(K.as_ptr().add(48)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);

        // Rounds 52-55
        tmp = vaddq_u32(msg1, vld1q_u32(K.as_ptr().add(52)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);

        // Rounds 56-59
        tmp = vaddq_u32(msg2, vld1q_u32(K.as_ptr().add(56)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);

        // Rounds 60-63
        tmp = vaddq_u32(msg3, vld1q_u32(K.as_ptr().add(60)));
        let tmp1 = state0;
        state0 = vsha256hq_u32(state0, state1, tmp);
        state1 = vsha256h2q_u32(state1, tmp1, tmp);

        // Add saved state
        state0 = vaddq_u32(state0, state0_save);
        state1 = vaddq_u32(state1, state1_save);

        // Store result (big-endian)
        let mut result = [0u8; 32];
        vst1q_u8(result.as_mut_ptr(), vrev32q_u8(vreinterpretq_u8_u32(state0)));
        vst1q_u8(result.as_mut_ptr().add(16), vrev32q_u8(vreinterpretq_u8_u32(state1)));
        result
    }
}

/// Software SHA-256 compression fallback
#[cfg(not(all(target_arch = "aarch64", target_feature = "sha2")))]
#[inline(always)]
fn sha256_compress_soft(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
    // Convert state to u32 words (SHA-256 internal state)
    let mut hash_state = [0u32; 8];
    for i in 0..8 {
        hash_state[i] = u32::from_be_bytes([
            state[i * 4],
            state[i * 4 + 1],
            state[i * 4 + 2],
            state[i * 4 + 3],
        ]);
    }

    // Prepare message block
    let mut msg_block = [0u8; 64];
    msg_block.copy_from_slice(block);

    // Apply SHA-256 compression function
    sha2::compress256(&mut hash_state, &[msg_block.into()]);

    // Convert back to bytes
    let mut result = [0u8; 32];
    for i in 0..8 {
        let bytes = hash_state[i].to_be_bytes();
        result[i * 4..i * 4 + 4].copy_from_slice(&bytes);
    }

    result
}

/// BLAKE3 compression function (7 rounds)
///
/// Implements the core BLAKE3 compression with 7 rounds as specified
#[inline(always)]
pub fn blake3_compress(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
    // BLAKE3 constants (first 8 words of fractional part of sqrt of first 8 primes)
    const IV: [u32; 8] = [
        0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
        0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
    ];

    // Message permutation schedule for BLAKE3
    const MSG_SCHEDULE: [[usize; 16]; 7] = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8],
        [3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1],
        [10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6],
        [12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4],
        [9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7],
        [11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13],
    ];

    // Convert state to words
    let mut h = [0u32; 8];
    for i in 0..8 {
        h[i] = u32::from_le_bytes([
            state[i * 4],
            state[i * 4 + 1],
            state[i * 4 + 2],
            state[i * 4 + 3],
        ]);
    }

    // Convert block to message words
    let mut m = [0u32; 16];
    for i in 0..16 {
        m[i] = u32::from_le_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }

    // Initialize state matrix
    let mut v = [0u32; 16];
    v[0..8].copy_from_slice(&h);
    v[8..16].copy_from_slice(&IV);

    // 7 rounds of mixing
    for s in &MSG_SCHEDULE[..7] {

        // Column mixing
        g(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
        g(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
        g(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
        g(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);

        // Diagonal mixing
        g(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
        g(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        g(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
        g(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
    }

    // Finalize: XOR the two halves
    for i in 0..8 {
        h[i] = v[i] ^ v[i + 8];
    }

    // Convert back to bytes
    let mut result = [0u8; 32];
    for i in 0..8 {
        let bytes = h[i].to_le_bytes();
        result[i * 4..i * 4 + 4].copy_from_slice(&bytes);
    }

    result
}

/// BLAKE3 G mixing function
#[inline(always)]
fn g(v: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, mx: u32, my: u32) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(mx);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(12);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(my);
    v[d] = (v[d] ^ v[a]).rotate_right(8);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(7);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_compress_deterministic() {
        let state = [0u8; 32];
        let block = [1u8; 64];

        let result1 = aes_compress(&state, &block);
        let result2 = aes_compress(&state, &block);

        assert_eq!(result1, result2);
        assert_ne!(result1, state); // Should be different from input
    }

    #[test]
    fn test_sha256_compress_deterministic() {
        let state = [0u8; 32];
        let block = [1u8; 64];

        let result1 = sha256_compress(&state, &block);
        let result2 = sha256_compress(&state, &block);

        assert_eq!(result1, result2);
        assert_ne!(result1, state);
    }

    #[test]
    fn test_blake3_compress_deterministic() {
        let state = [0u8; 32];
        let block = [1u8; 64];

        let result1 = blake3_compress(&state, &block);
        let result2 = blake3_compress(&state, &block);

        assert_eq!(result1, result2);
        assert_ne!(result1, state);
    }
}
