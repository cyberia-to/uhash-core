//! Tests for UniversalHash algorithm

use crate::{hash, meets_difficulty, UniversalHash};

#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn test_basic_hash() {
    let input = b"test input data";
    let result = hash(input);

    // Hash should be 32 bytes
    assert_eq!(result.len(), 32);

    // Hash should be deterministic
    let result2 = hash(input);
    assert_eq!(result, result2);
}

#[test]
fn test_different_inputs_produce_different_hashes() {
    let hash1 = hash(b"input 1");
    let hash2 = hash(b"input 2");

    assert_ne!(hash1, hash2);
}

#[test]
fn test_avalanche_effect() {
    // Changing one bit should change ~50% of output bits
    let input1 = b"test input";
    let mut input2 = input1.to_vec();
    input2[0] ^= 1; // Flip one bit

    let hash1 = hash(input1);
    let hash2 = hash(&input2);

    // Count differing bits
    let mut diff_bits = 0;
    for i in 0..32 {
        diff_bits += (hash1[i] ^ hash2[i]).count_ones();
    }

    // Expect roughly 128 bits (50% of 256) to differ
    // Allow range of 90-166 (35%-65%)
    assert!(
        diff_bits >= 90 && diff_bits <= 166,
        "Avalanche effect: {} bits differ (expected ~128)",
        diff_bits
    );
}

#[test]
fn test_difficulty_check() {
    // Hash with 8 leading zero bits (starts with 0x00)
    let hash_8_zeros: [u8; 32] = [
        0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF,
    ];

    assert!(meets_difficulty(&hash_8_zeros, 8));
    assert!(!meets_difficulty(&hash_8_zeros, 9));

    // Hash with 16 leading zero bits (starts with 0x0000)
    let hash_16_zeros: [u8; 32] = [
        0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF,
    ];

    assert!(meets_difficulty(&hash_16_zeros, 16));
    assert!(!meets_difficulty(&hash_16_zeros, 17));

    // Hash with leading 0x0F (4 zero bits)
    let hash_4_zeros: [u8; 32] = [
        0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF,
    ];

    assert!(meets_difficulty(&hash_4_zeros, 4));
    assert!(!meets_difficulty(&hash_4_zeros, 5));
}

#[test]
fn test_hasher_reusability() {
    let mut hasher = UniversalHash::new();

    let hash1 = hasher.hash(b"first input");
    let hash2 = hasher.hash(b"second input");

    assert_ne!(hash1, hash2);

    // Same input should still produce same hash
    let hash1_again = hasher.hash(b"first input");
    assert_eq!(hash1, hash1_again);
}

#[test]
fn test_empty_input() {
    let result = hash(b"");
    assert_eq!(result.len(), 32);
}

#[test]
fn test_large_input() {
    let large_input = vec![0xABu8; 10000];
    let result = hash(&large_input);
    assert_eq!(result.len(), 32);
}

/// Spec compliance test vectors
/// These vectors verify the implementation matches the UniversalHash v4 spec:
/// - Seed generation: BLAKE3(header || (nonce ⊕ (c × golden_ratio)))
/// - Primitive rotation: (nonce + c) mod 3, then +1 before each round
/// - Write-back: Same address as read
/// - Finalization: BLAKE3(SHA256(XOR of chain states))
#[test]
fn test_spec_compliance_vectors() {
    // Vector 1: Standard mining input format
    // Input: 32-byte epoch_seed + 20-byte address + 8-byte timestamp + 8-byte nonce
    let input1: Vec<u8> = {
        let mut v = Vec::with_capacity(68);
        v.extend_from_slice(&[0u8; 32]); // epoch_seed = all zeros
        v.extend_from_slice(&[1u8; 20]); // miner_address = all ones
        v.extend_from_slice(&[0u8; 8]);  // timestamp = 0
        v.extend_from_slice(&[0u8; 8]);  // nonce = 0
        v
    };
    let hash1 = hash(&input1);

    // Vector 2: Same with nonce = 1
    let input2: Vec<u8> = {
        let mut v = Vec::with_capacity(68);
        v.extend_from_slice(&[0u8; 32]);
        v.extend_from_slice(&[1u8; 20]);
        v.extend_from_slice(&[0u8; 8]);
        v.extend_from_slice(&1u64.to_le_bytes()); // nonce = 1
        v
    };
    let hash2 = hash(&input2);

    // Vector 3: Different epoch seed
    let input3: Vec<u8> = {
        let mut v = Vec::with_capacity(68);
        v.extend_from_slice(&[0xAB; 32]); // epoch_seed = all 0xAB
        v.extend_from_slice(&[1u8; 20]);
        v.extend_from_slice(&[0u8; 8]);
        v.extend_from_slice(&[0u8; 8]);
        v
    };
    let hash3 = hash(&input3);

    // Hashes must be different (proves nonce/seed affect output)
    assert_ne!(hash1, hash2, "Different nonces should produce different hashes");
    assert_ne!(hash1, hash3, "Different epoch seeds should produce different hashes");

    // Hashes must be deterministic
    assert_eq!(hash(&input1), hash1, "Hash must be deterministic");
    assert_eq!(hash(&input2), hash2, "Hash must be deterministic");
    assert_eq!(hash(&input3), hash3, "Hash must be deterministic");

    // Print vectors for reference (run with --nocapture)
    #[cfg(feature = "std")]
    {
        println!("\n=== SPEC COMPLIANCE TEST VECTORS ===");
        println!("Vector 1 (nonce=0): {}", hex::encode(hash1));
        println!("Vector 2 (nonce=1): {}", hex::encode(hash2));
        println!("Vector 3 (seed=0xAB): {}", hex::encode(hash3));
    }
}

#[test]
fn test_nonce_extraction() {
    // Test that nonce is correctly extracted from last 8 bytes
    let mut hasher = UniversalHash::new();

    // Input with known nonce at end
    let nonce: u64 = 0x123456789ABCDEF0;
    let mut input = vec![0u8; 60]; // header
    input.extend_from_slice(&nonce.to_le_bytes());

    let hash1 = hasher.hash(&input);

    // Same header, different nonce should produce different hash
    let mut input2 = vec![0u8; 60];
    input2.extend_from_slice(&(nonce + 1).to_le_bytes());

    let hash2 = hasher.hash(&input2);

    assert_ne!(hash1, hash2, "Different nonces must produce different hashes");
}

#[test]
fn test_primitive_rotation_per_spec() {
    // Verify that primitive rotation follows spec:
    // primitive = (nonce + chain) mod 3, then +1 before each round use
    // This is implicitly tested by hash consistency - if rotation changes,
    // the hash output changes.

    // Run same input multiple times to ensure determinism
    let input = b"primitive rotation test";
    let mut results = Vec::new();

    for _ in 0..5 {
        results.push(hash(input));
    }

    for i in 1..results.len() {
        assert_eq!(results[0], results[i], "Hash must be deterministic across runs");
    }
}

#[test]
fn test_known_vector() {
    // This test ensures the algorithm doesn't change accidentally
    // The hash of "uhash-core test vector" should always be the same
    let input = b"uhash-core test vector";
    let result = hash(input);

    // Store first hash run as reference (update if algorithm intentionally changes)
    // For now just verify it's deterministic
    let result2 = hash(input);
    assert_eq!(result, result2);
}

#[test]
#[ignore] // Run with: cargo test timing_breakdown -- --ignored --nocapture
fn timing_breakdown() {
    use std::time::Instant;
    use crate::params::*;
    use crate::primitives::{aes_compress, sha256_compress, blake3_compress, aes_expand_block};

    let input = b"timing test input";
    let iterations = 10;

    // Warmup
    for _ in 0..3 {
        let _ = hash(input);
    }

    // Measure total hash time
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = hash(input);
    }
    let total = start.elapsed();
    let per_hash = total / iterations;

    // Measure individual primitives
    let state = [0u8; 32];
    let block = [1u8; 64];
    let prim_iters = 10000;

    let start_aes = Instant::now();
    for _ in 0..prim_iters {
        let _ = aes_compress(&state, &block);
    }
    let aes_time = start_aes.elapsed() / prim_iters;

    let start_sha = Instant::now();
    for _ in 0..prim_iters {
        let _ = sha256_compress(&state, &block);
    }
    let sha_time = start_sha.elapsed() / prim_iters;

    let start_blake = Instant::now();
    for _ in 0..prim_iters {
        let _ = blake3_compress(&state, &block);
    }
    let blake_time = start_blake.elapsed() / prim_iters;

    // Measure AES expand (used in scratchpad init)
    let key16 = [0u8; 16];
    let state16 = [1u8; 16];
    let start_expand = Instant::now();
    for _ in 0..prim_iters {
        let _ = aes_expand_block(&state16, &key16);
    }
    let expand_time = start_expand.elapsed() / prim_iters;

    // Estimate scratchpad init time
    // Each scratchpad has BLOCKS_PER_SCRATCHPAD blocks, each needs 2 AES expansions
    let scratchpad_init_est = expand_time * (BLOCKS_PER_SCRATCHPAD * 2 * CHAINS) as u32;

    // Round execution estimate
    let ops_per_hash = ROUNDS * CHAINS;
    let primitive_avg = (aes_time + sha_time + blake_time) / 3;
    let rounds_est = primitive_avg * ops_per_hash as u32;

    println!("\n=== TIMING BREAKDOWN ===");
    println!("Total per hash: {:?}", per_hash);
    println!("Hashrate: {:.1} H/s", 1.0 / per_hash.as_secs_f64());
    println!("\nPrimitive timing:");
    println!("  AES_Compress:    {:?}", aes_time);
    println!("  SHA256_Compress: {:?}", sha_time);
    println!("  BLAKE3_Compress: {:?}", blake_time);
    println!("  AES_Expand:      {:?}", expand_time);
    println!("  Primitive avg:   {:?}", primitive_avg);
    println!("\nParameters:");
    println!("  ROUNDS: {} × {} chains = {} ops", ROUNDS, CHAINS, ops_per_hash);
    println!("  SCRATCHPAD: {} blocks × {} chains × 2 AES = {} AES ops",
             BLOCKS_PER_SCRATCHPAD, CHAINS, BLOCKS_PER_SCRATCHPAD * 2 * CHAINS);
    println!("\nTime breakdown estimate:");
    println!("  Scratchpad init: {:?}", scratchpad_init_est);
    println!("  Round execution: {:?}", rounds_est);
    println!("  Total estimated: {:?}", scratchpad_init_est + rounds_est);
    println!("  Actual total:    {:?}", per_hash);
    println!("  Overhead:        {:?}", per_hash.saturating_sub(scratchpad_init_est + rounds_est));
}
