//! UniversalHash v4 Algorithm Parameters
//!
//! These parameters are tuned for democratic mining where phones
//! can compete meaningfully with desktops (1:3-5 ratio).

/// Number of parallel computation chains
pub const CHAINS: usize = 4;

/// Scratchpad size per chain in bytes (512 KB)
pub const SCRATCHPAD_SIZE: usize = 512 * 1024;

/// Total memory footprint (2 MB)
pub const TOTAL_MEMORY: usize = CHAINS * SCRATCHPAD_SIZE;

/// Number of rounds per chain (spec: 12,288)
pub const ROUNDS: usize = 12_288;

/// Block size in bytes for memory operations
pub const BLOCK_SIZE: usize = 64;

/// Number of blocks per scratchpad
pub const BLOCKS_PER_SCRATCHPAD: usize = SCRATCHPAD_SIZE / BLOCK_SIZE;

/// AES block size
pub const AES_BLOCK_SIZE: usize = 16;

/// SHA-256 output size
pub const SHA256_SIZE: usize = 32;

/// BLAKE3 output size
pub const BLAKE3_SIZE: usize = 32;

/// Algorithm version
pub const VERSION: u8 = 4;
