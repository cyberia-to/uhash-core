use wasm_bindgen::prelude::*;
use uhash_core::UniversalHash;

#[wasm_bindgen]
pub struct Benchmark {
    hasher: UniversalHash,
}

#[wasm_bindgen]
impl Benchmark {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            hasher: UniversalHash::new(),
        }
    }

    /// Run benchmark with specified number of hashes
    /// Returns hashrate in H/s
    #[wasm_bindgen]
    pub fn run(&mut self, num_hashes: u32) -> f64 {
        let window = web_sys::window().unwrap();
        let performance = window.performance().unwrap();

        let start = performance.now();

        for i in 0..num_hashes {
            let input = format!("benchmark_input_{}", i);
            let _ = self.hasher.hash(input.as_bytes());
        }

        let end = performance.now();
        let elapsed_ms = end - start;
        let elapsed_s = elapsed_ms / 1000.0;

        (num_hashes as f64) / elapsed_s
    }

    /// Get algorithm parameters as JSON string
    #[wasm_bindgen]
    pub fn get_params(&self) -> String {
        format!(
            r#"{{"chains": {}, "scratchpad_kb": {}, "total_mb": {}, "rounds": {}}}"#,
            uhash_core::CHAINS,
            uhash_core::SCRATCHPAD_SIZE / 1024,
            uhash_core::TOTAL_MEMORY / (1024 * 1024),
            uhash_core::ROUNDS
        )
    }
}

/// Single hash function for testing
#[wasm_bindgen]
pub fn hash_once(input: &[u8]) -> Vec<u8> {
    uhash_core::hash(input).to_vec()
}
