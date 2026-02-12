//! C FFI bindings for mobile platforms

use crate::UniversalHash;
use core::slice;

/// Opaque hasher handle for FFI
pub struct UHasher {
    inner: UniversalHash,
}

/// Create a new hasher instance
/// Returns a pointer to the hasher (caller must free with uhash_free)
#[no_mangle]
pub extern "C" fn uhash_new() -> *mut UHasher {
    let hasher = Box::new(UHasher {
        inner: UniversalHash::new(),
    });
    Box::into_raw(hasher)
}

/// Free a hasher instance
#[no_mangle]
pub extern "C" fn uhash_free(hasher: *mut UHasher) {
    if !hasher.is_null() {
        unsafe {
            let _ = Box::from_raw(hasher);
        }
    }
}

/// Compute hash of input data
/// - hasher: pointer from uhash_new()
/// - input: pointer to input bytes
/// - input_len: length of input
/// - output: pointer to 32-byte buffer for result
#[no_mangle]
pub extern "C" fn uhash_hash(
    hasher: *mut UHasher,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
) {
    if hasher.is_null() || input.is_null() || output.is_null() {
        return;
    }

    unsafe {
        let hasher = &mut *hasher;
        let input_slice = slice::from_raw_parts(input, input_len);
        let result = hasher.inner.hash(input_slice);

        let output_slice = slice::from_raw_parts_mut(output, 32);
        output_slice.copy_from_slice(&result);
    }
}

/// Benchmark: compute N hashes and return total microseconds
#[no_mangle]
pub extern "C" fn uhash_benchmark(iterations: u32) -> u64 {
    use std::time::Instant;

    let mut hasher = UniversalHash::new();
    let input = b"benchmark test input data for mobile";

    let start = Instant::now();
    for i in 0..iterations {
        let mut data = input.to_vec();
        data.extend_from_slice(&i.to_le_bytes());
        let _ = hasher.hash(&data);
    }
    let elapsed = start.elapsed();

    elapsed.as_micros() as u64
}

/// Get hash rate (hashes per second) from a benchmark run
#[no_mangle]
pub extern "C" fn uhash_hashrate(iterations: u32, microseconds: u64) -> f64 {
    if microseconds == 0 {
        return 0.0;
    }
    (iterations as f64) / (microseconds as f64 / 1_000_000.0)
}
