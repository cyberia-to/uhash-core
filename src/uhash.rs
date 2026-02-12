//! Core UniversalHash v4 Implementation (Spec-Compliant)
//!
//! The algorithm uses 4 parallel chains with 512KB scratchpads each.
//! Each chain performs rounds using raw compression functions (AES, SHA-256, BLAKE3)
//! in a sequential pattern that prevents GPU parallelism.
//!
//! This implementation follows the spec exactly:
//! - Seed generation: BLAKE3(header || (nonce ⊕ (c × golden_ratio)))
//! - Primitive rotation: (nonce + c) mod 3, then +1 before each round
//! - Write-back: Same address as read (not computed from new state)
//! - No cross-chain mixing (spec doesn't specify it)

#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use blake3::Hasher as Blake3;
use sha2::{Digest, Sha256};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::params::*;
use crate::primitives::{aes_compress, blake3_compress, sha256_compress};

/// Mask for address calculation (BLOCKS_PER_SCRATCHPAD - 1)
/// Since BLOCKS_PER_SCRATCHPAD = 8192 = 2^13, this is 0x1FFF
const ADDRESS_MASK: usize = BLOCKS_PER_SCRATCHPAD - 1;

/// Golden ratio constant for seed generation (Fibonacci hashing constant)
const GOLDEN_RATIO: u64 = 0x9E3779B97F4A7C15;

/// UniversalHash v4 hasher
///
/// This struct maintains the scratchpads and chain states needed for hashing.
/// It can be reused for multiple hashes to avoid repeated allocations.
pub struct UniversalHash {
    /// 4 scratchpads, one per chain (512KB each)
    scratchpads: Vec<Vec<u8>>,
    /// Current state for each chain
    chain_states: [[u8; 32]; CHAINS],
    /// Effective nonce extracted from input (last 8 bytes)
    effective_nonce: u64,
}

impl UniversalHash {
    /// Create a new UniversalHash instance
    ///
    /// Allocates 2MB of memory for the scratchpads.
    pub fn new() -> Self {
        Self {
            scratchpads: vec![vec![0u8; SCRATCHPAD_SIZE]; CHAINS],
            chain_states: [[0u8; 32]; CHAINS],
            effective_nonce: 0,
        }
    }

    /// Compute the UniversalHash of input data
    ///
    /// The input should be formatted as:
    /// `epoch_seed || miner_address || timestamp || nonce`
    /// where nonce is the last 8 bytes.
    ///
    /// Returns a 32-byte hash.
    pub fn hash(&mut self, input: &[u8]) -> [u8; 32] {
        // Extract effective nonce from last 8 bytes of input (or hash if shorter)
        self.effective_nonce = extract_nonce(input);

        // Phase 1: Initialize scratchpads using input (spec-compliant seed generation)
        self.init_scratchpads(input);

        // Phase 2: Execute main mixing rounds (spec-compliant, no cross-chain mixing)
        self.execute_rounds();

        // Phase 3: Finalize and produce output
        self.finalize()
    }

    /// Initialize all scratchpads from input using expansion
    /// Spec: seed[c] = BLAKE3_256(header || (nonce ⊕ (c × golden_ratio)))
    #[cfg(feature = "parallel")]
    fn init_scratchpads(&mut self, input: &[u8]) {
        let nonce = self.effective_nonce;

        // Pre-compute all chain seeds using BLAKE3 with XORed nonce per spec
        let mut chain_seeds: [[u8; 32]; CHAINS] = [[0u8; 32]; CHAINS];
        for (chain, (seed, state)) in chain_seeds
            .iter_mut()
            .zip(self.chain_states.iter_mut())
            .enumerate()
        {
            // Spec: nonce ⊕ (c × golden_ratio)
            let offset = (chain as u64).wrapping_mul(GOLDEN_RATIO);
            let modified_nonce = nonce ^ offset;

            // Spec: BLAKE3(header || modified_nonce)
            // Header is input without last 8 bytes (nonce)
            let header_len = input.len().saturating_sub(8);
            let mut hasher = Blake3::new();
            hasher.update(&input[..header_len]);
            hasher.update(&modified_nonce.to_le_bytes());
            let hash = hasher.finalize();
            seed.copy_from_slice(hash.as_bytes());
            state.copy_from_slice(hash.as_bytes());
        }

        // Fill scratchpads in parallel
        self.scratchpads
            .par_iter_mut()
            .zip(chain_seeds.par_iter())
            .for_each(|(scratchpad, seed)| {
                fill_scratchpad_aes(scratchpad, seed);
            });
    }

    /// Initialize all scratchpads from input using expansion (sequential fallback)
    /// Spec: seed[c] = BLAKE3_256(header || (nonce ⊕ (c × golden_ratio)))
    #[cfg(not(feature = "parallel"))]
    fn init_scratchpads(&mut self, input: &[u8]) {
        let nonce = self.effective_nonce;
        let header_len = input.len().saturating_sub(8);

        for (chain, state) in self.chain_states.iter_mut().enumerate() {
            // Spec: nonce ⊕ (c × golden_ratio)
            let offset = (chain as u64).wrapping_mul(GOLDEN_RATIO);
            let modified_nonce = nonce ^ offset;

            // Spec: BLAKE3(header || modified_nonce)
            let mut hasher = Blake3::new();
            hasher.update(&input[..header_len]);
            hasher.update(&modified_nonce.to_le_bytes());
            let hash = hasher.finalize();

            state.copy_from_slice(hash.as_bytes());

            // Fill scratchpad using AES-based expansion
            let seed_array: [u8; 32] = hash.as_bytes().try_into().unwrap();
            fill_scratchpad_aes(&mut self.scratchpads[chain], &seed_array);
        }
    }

    /// Execute the main mixing rounds (spec-compliant: no cross-chain mixing)
    #[cfg(feature = "parallel")]
    fn execute_rounds(&mut self) {
        let nonce = self.effective_nonce;

        // Process all chains in parallel - each chain runs all rounds independently
        // Spec does NOT specify cross-chain mixing, so we don't do it
        self.scratchpads
            .par_iter_mut()
            .zip(self.chain_states.par_iter_mut())
            .enumerate()
            .for_each(|(chain, (scratchpad, state))| {
                // Spec: primitive = (nonce + c) mod 3
                let initial_primitive = ((nonce as usize) + chain) % 3;

                // Execute all rounds for this chain
                for round in 0..ROUNDS {
                    round_step_spec_compliant(scratchpad, state, initial_primitive, round);
                }
            });
    }

    /// Execute the main mixing rounds (sequential fallback, spec-compliant)
    #[cfg(not(feature = "parallel"))]
    fn execute_rounds(&mut self) {
        let nonce = self.effective_nonce;

        // Process each chain independently (spec-compliant: no cross-chain mixing)
        for chain in 0..CHAINS {
            // Spec: primitive = (nonce + c) mod 3
            let initial_primitive = ((nonce as usize) + chain) % 3;

            // Execute all rounds for this chain
            for round in 0..ROUNDS {
                round_step_spec_compliant(
                    &mut self.scratchpads[chain],
                    &mut self.chain_states[chain],
                    initial_primitive,
                    round,
                );
            }
        }
    }

    /// Finalize and produce the 32-byte output hash per spec
    /// Spec: result = BLAKE3_256(SHA256_256(combined))
    fn finalize(&self) -> [u8; 32] {
        // XOR all chain states together
        let mut combined = [0u8; 32];
        for state in &self.chain_states {
            for i in 0..32 {
                combined[i] ^= state[i];
            }
        }

        // Double hash: SHA256 then BLAKE3 (per spec)
        let sha_hash = Sha256::digest(combined);
        let mut hasher = Blake3::new();
        hasher.update(&sha_hash);
        hasher.finalize().into()
    }
}

/// Extract nonce from input (last 8 bytes, or hash if shorter)
#[inline(always)]
fn extract_nonce(input: &[u8]) -> u64 {
    if input.len() >= 8 {
        // Use last 8 bytes as nonce
        let nonce_bytes: [u8; 8] = input[input.len() - 8..].try_into().unwrap();
        u64::from_le_bytes(nonce_bytes)
    } else {
        // For short inputs, hash to get a nonce
        let hash = blake3::hash(input);
        let bytes: [u8; 8] = hash.as_bytes()[..8].try_into().unwrap();
        u64::from_le_bytes(bytes)
    }
}

/// Fill a scratchpad using AES-based expansion per spec
/// Spec:
///   key = seed[0:16]
///   state = seed[16:32]
///   For i = 0 to NUM_BLOCKS - 1:
///     state = AES_4Rounds(state, key)
///     scratchpad[i × 64 : (i+1) × 64] = state || AES_4Rounds(state, key)
#[inline(always)]
fn fill_scratchpad_aes(scratchpad: &mut [u8], seed: &[u8; 32]) {
    use crate::primitives::aes_expand_block;

    let key: [u8; 16] = seed[0..16].try_into().unwrap();
    let mut state: [u8; 16] = seed[16..32].try_into().unwrap();

    for i in 0..BLOCKS_PER_SCRATCHPAD {
        // Apply 4 AESENC rounds (per spec)
        state = aes_expand_block(&state, &key);
        let offset = i * BLOCK_SIZE;

        // First 16 bytes: state after first AES
        scratchpad[offset..offset + 16].copy_from_slice(&state);

        // Next 16 bytes: state after second AES (per spec)
        let state2 = aes_expand_block(&state, &key);
        scratchpad[offset + 16..offset + 32].copy_from_slice(&state2);

        // Remaining 32 bytes: duplicate first 32 bytes
        // (spec says 32 bytes per block but BLOCK_SIZE is 64)
        scratchpad[offset + 32..offset + 48].copy_from_slice(&state);
        scratchpad[offset + 48..offset + 64].copy_from_slice(&state2);
    }
}

/// Single round step for one chain (spec-compliant version)
///
/// Spec:
/// - Address: computed from current state
/// - Primitive: (initial_primitive + round + 1) mod 3  (increment BEFORE use)
/// - Write-back: SAME address as read (not new address)
#[inline(always)]
fn round_step_spec_compliant(
    scratchpad: &mut [u8],
    state: &mut [u8; 32],
    initial_primitive: usize,
    round: usize,
) {
    // Compute memory address from state per spec formula
    let addr = compute_address(state, round);

    // Read block from scratchpad
    // SAFETY: addr is always within bounds due to ADDRESS_MASK
    let block: [u8; BLOCK_SIZE] = unsafe {
        core::ptr::read(scratchpad.as_ptr().add(addr) as *const [u8; BLOCK_SIZE])
    };

    // Spec: primitive = (primitive + 1) mod 3 BEFORE applying
    // Where primitive starts at (nonce + chain) mod 3
    // So at round r: primitive = (initial_primitive + r + 1) mod 3
    let primitive = (initial_primitive + round + 1) % 3;

    // Apply raw compression function based on primitive
    let new_state = match primitive {
        0 => aes_compress(state, &block),
        1 => sha256_compress(state, &block),
        _ => blake3_compress(state, &block),
    };

    // Spec: Write back to SAME address as read (not computed from new_state!)
    // SAFETY: addr is always within bounds due to ADDRESS_MASK
    unsafe {
        core::ptr::copy_nonoverlapping(new_state.as_ptr(), scratchpad.as_mut_ptr().add(addr), 32);
    }

    // Update chain state
    *state = new_state;
}

/// Compute scratchpad address from state per spec
/// Spec: mixed = state[0:8] ⊕ state[8:16] ⊕ rotl64(round, 13) ⊕ (round × 0x517cc1b727220a95)
///       addr = (mixed mod NUM_BLOCKS) × BLOCK_SIZE
#[inline(always)]
fn compute_address(state: &[u8; 32], round: usize) -> usize {
    const MIXING_CONSTANT: u64 = 0x517cc1b727220a95;

    // Read u64s directly using pointer reads (faster than try_into)
    // SAFETY: state is 32 bytes, reading at offsets 0 and 8 is safe
    let state_lo = unsafe { core::ptr::read_unaligned(state.as_ptr() as *const u64) };
    let state_hi = unsafe { core::ptr::read_unaligned(state.as_ptr().add(8) as *const u64) };
    let round_u64 = round as u64;

    // Spec formula for unpredictable address
    let mixed = state_lo
        ^ state_hi
        ^ round_u64.rotate_left(13)
        ^ round_u64.wrapping_mul(MIXING_CONSTANT);

    // Use bitwise AND instead of modulo (NUM_BLOCKS is power of 2)
    ((mixed as usize) & ADDRESS_MASK) * BLOCK_SIZE
}

impl Default for UniversalHash {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function for single-shot hashing
///
/// Creates a new hasher, computes the hash, and returns it.
/// For multiple hashes, prefer creating a `UniversalHash` instance
/// and reusing it to avoid repeated memory allocation.
pub fn hash(input: &[u8]) -> [u8; 32] {
    let mut hasher = UniversalHash::new();
    hasher.hash(input)
}

/// Check if a hash meets the required difficulty
///
/// Difficulty is measured as the number of leading zero bits required.
/// For example, difficulty 16 requires the first 2 bytes to be zero.
///
/// # Example
///
/// ```rust
/// use uhash_core::meets_difficulty;
///
/// // Hash with 20 leading zero bits (0x00, 0x00, 0x0F = 16 + 4 zeros)
/// let hash: [u8; 32] = [
///     0x00, 0x00, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
///     0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
///     0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
///     0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/// ];
/// assert!(meets_difficulty(&hash, 16));  // 16 leading zeros - pass
/// assert!(meets_difficulty(&hash, 20));  // 20 leading zeros - pass
/// assert!(!meets_difficulty(&hash, 21)); // Only 20 zeros - fail
/// ```
#[inline(always)]
pub fn meets_difficulty(hash: &[u8; 32], difficulty: u32) -> bool {
    let mut zero_bits = 0u32;

    for byte in hash.iter() {
        if *byte == 0 {
            zero_bits += 8;
        } else {
            zero_bits += byte.leading_zeros();
            break;
        }
    }

    zero_bits >= difficulty
}
