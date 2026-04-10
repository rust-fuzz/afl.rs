// Demonstrates how `fuzz_with_reset!` improves AFL++ persistent mode stability
// when using static state.
//
// Setup:
//   `cargo run -p cargo-afl -- afl build --example reset_demo --manifest-path afl/Cargo.toml`
//   `mkdir -p /tmp/afl-input && echo "test" > /tmp/afl-input/seed`
//
// Without reset (low stability):
//   `AFL_NO_UI=1 cargo run -p cargo-afl -- afl fuzz \
//     -i /tmp/afl-input -o /tmp/afl-out-bad -V 15 target/debug/examples/reset_demo`
//
// With reset (high stability):
//   `USE_RESET=1 AFL_NO_UI=1 cargo run -p cargo-afl -- afl fuzz \
//     -i /tmp/afl-input -o /tmp/afl-out-reset -V 15 target/debug/examples/reset_demo`
//
// Compare stability:
//   `grep stability /tmp/afl-out-bad/default/fuzzer_stats /tmp/afl-out-reset/default/fuzzer_stats`

use std::sync::Mutex;

static CACHE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

fn main() {
    if std::env::var("USE_RESET").is_ok() {
        afl::fuzz_with_reset!(|data: &[u8]| { fuzz_body(data) }, || {
            *CACHE.lock().unwrap() = None;
        });
    } else {
        afl::fuzz!(|data: &[u8]| {
            fuzz_body(data);
        });
    }
}

fn fuzz_body(data: &[u8]) {
    let mut cache = CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(data.to_vec());
    }
    drop(cache);
    assert!(!(data.len() > 2 && data[0] == b'x'), "crash");
}
