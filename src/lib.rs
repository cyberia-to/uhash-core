//! # UniversalHash Core Algorithm
//!
//! A democratic proof-of-work algorithm designed for fair mining where
//! smartphones can meaningfully compete with servers.
//!
//! **v0.2.0** - Full spec compliance with UniversalHash v4 specification.
//!
//! ## Features
//!
//! - **Spec-Compliant**: Implements UniversalHash v4 specification exactly
//! - **Democratic Mining**: Phone-to-desktop ratio of 1:3-5
//! - **ASIC Resistance**: Multi-primitive design (AES + SHA-256 + BLAKE3)
//! - **Memory-Hard**: 2MB scratchpad prevents GPU parallelism
//!
//! ## Algorithm Parameters (v4)
//!
//! - 4 parallel computation chains
//! - 512KB scratchpad per chain (2MB total)
//! - 12,288 rounds per chain
//! - Triple primitive rotation: AES, SHA-256, BLAKE3
//!
//! ## Input Format
//!
//! The algorithm extracts the nonce from the **last 8 bytes** of input:
//!
//! ```text
//! input = header || nonce
//!         ^^^^^^    ^^^^^
//!         any len   8 bytes (little-endian u64)
//! ```
//!
//! Typical mining format: `epoch_seed (32B) || miner_address (20B) || timestamp (8B) || nonce (8B)`
//!
//! ## Example
//!
//! ```rust
//! use uhash_core::{UniversalHash, hash, meets_difficulty};
//!
//! // Single-shot hashing
//! let result = hash(b"input data");
//!
//! // Check difficulty (leading zero bits)
//! if meets_difficulty(&result, 16) {
//!     println!("Found hash with 16+ leading zero bits!");
//! }
//!
//! // Reusable hasher (avoids re-allocation)
//! let mut hasher = UniversalHash::new();
//! let hash1 = hasher.hash(b"first");
//! let hash2 = hasher.hash(b"second");
//! ```
//!
//! ## no_std Support
//!
//! This crate supports `no_std` environments with the `alloc` crate:
//!
//! ```toml
//! [dependencies]
//! uhash-core = { version = "0.2", default-features = false }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod params;
mod primitives;
mod uhash;

#[cfg(feature = "std")]
mod ffi;

pub use params::*;
pub use uhash::{hash, meets_difficulty, UniversalHash};

#[cfg(test)]
mod tests;
