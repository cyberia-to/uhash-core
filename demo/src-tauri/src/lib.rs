use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::State;
use uhash_core::UniversalHash;

// Shared state
struct AppState {
    hasher: Mutex<UniversalHash>,
    mining: AtomicBool,
    hash_count: AtomicU64,
    start_time: Mutex<Option<Instant>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            hasher: Mutex::new(UniversalHash::new()),
            mining: AtomicBool::new(false),
            hash_count: AtomicU64::new(0),
            start_time: Mutex::new(None),
        }
    }
}

#[tauri::command]
fn get_params() -> serde_json::Value {
    serde_json::json!({
        "chains": uhash_core::CHAINS,
        "scratchpad_kb": uhash_core::SCRATCHPAD_SIZE / 1024,
        "total_mb": uhash_core::TOTAL_MEMORY / (1024 * 1024),
        "rounds": uhash_core::ROUNDS,
        "block_size": uhash_core::BLOCK_SIZE
    })
}

#[tauri::command]
fn benchmark(count: u32, state: State<Arc<AppState>>) -> serde_json::Value {
    let mut hasher = state.hasher.lock().unwrap();

    let start = Instant::now();

    for i in 0..count {
        let input = format!("benchmark_input_{}", i);
        let _ = hasher.hash(input.as_bytes());
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    let hashrate = count as f64 / elapsed.as_secs_f64();

    serde_json::json!({
        "count": count,
        "elapsed_ms": elapsed_ms,
        "hashrate": hashrate
    })
}

#[tauri::command]
fn start_mining(state: State<Arc<AppState>>) -> serde_json::Value {
    if state.mining.load(Ordering::SeqCst) {
        return serde_json::json!({ "success": false, "error": "Already mining" });
    }

    state.mining.store(true, Ordering::SeqCst);
    state.hash_count.store(0, Ordering::SeqCst);
    *state.start_time.lock().unwrap() = Some(Instant::now());

    let state_clone = state.inner().clone();

    std::thread::spawn(move || {
        let mut hasher = UniversalHash::new();
        let mut nonce: u64 = 0;

        while state_clone.mining.load(Ordering::SeqCst) {
            let input = format!("mining_nonce_{}", nonce);
            let _ = hasher.hash(input.as_bytes());
            nonce += 1;
            state_clone.hash_count.fetch_add(1, Ordering::SeqCst);
        }
    });

    serde_json::json!({ "success": true })
}

#[tauri::command]
fn stop_mining(state: State<Arc<AppState>>) -> serde_json::Value {
    state.mining.store(false, Ordering::SeqCst);

    let elapsed = state.start_time.lock().unwrap()
        .map(|t| t.elapsed().as_secs_f64())
        .unwrap_or(0.0);

    let count = state.hash_count.load(Ordering::SeqCst);
    let hashrate = if elapsed > 0.0 { count as f64 / elapsed } else { 0.0 };

    serde_json::json!({
        "success": true,
        "total_hashes": count,
        "elapsed_secs": elapsed,
        "avg_hashrate": hashrate
    })
}

#[tauri::command]
fn get_mining_status(state: State<Arc<AppState>>) -> serde_json::Value {
    let is_mining = state.mining.load(Ordering::SeqCst);
    let count = state.hash_count.load(Ordering::SeqCst);

    let elapsed = state.start_time.lock().unwrap()
        .map(|t| t.elapsed().as_secs_f64())
        .unwrap_or(0.0);

    let hashrate = if elapsed > 0.0 { count as f64 / elapsed } else { 0.0 };

    serde_json::json!({
        "mining": is_mining,
        "total_hashes": count,
        "elapsed_secs": elapsed,
        "hashrate": hashrate
    })
}

#[tauri::command]
fn single_hash(input: String, state: State<Arc<AppState>>) -> serde_json::Value {
    let mut hasher = state.hasher.lock().unwrap();

    let start = Instant::now();
    let hash = hasher.hash(input.as_bytes());
    let elapsed = start.elapsed();

    serde_json::json!({
        "hash": hex::encode(hash),
        "elapsed_ms": elapsed.as_secs_f64() * 1000.0
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![
            get_params,
            benchmark,
            single_hash,
            start_mining,
            stop_mining,
            get_mining_status
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
