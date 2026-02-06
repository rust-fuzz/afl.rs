// Demonstrates how fuzz_with_reset! improves AFL++ persistent mode stability
// when using static state.
//
// Without reset (low stability):
//   cargo run -p cargo-afl -- afl fuzz -i input -o /tmp/out target/debug/examples/reset_demo
//
// With reset (high stability):
//   USE_RESET=1 cargo run -p cargo-afl -- afl fuzz -i input -o /tmp/out target/debug/examples/reset_demo

use std::sync::Mutex;

static CACHE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

fn fuzz_body(data: &[u8]) {
    let mut cache = CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(data.to_vec());
    }
    drop(cache);
    if data.len() > 2 && data[0] == b'x' {
        panic!("crash");
    }
}

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
