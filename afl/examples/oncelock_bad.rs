// Demonstrates low stability in AFL++ persistent mode when using static state
// without resetting it. The Mutex<Option<...>> init path diverges between
// iteration 1 (None → Some) and iterations 2+ (already Some), causing AFL's
// stability to drop.

use std::sync::Mutex;

static CACHE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let mut cache = CACHE.lock().unwrap();
        if cache.is_none() {
            *cache = Some(data.to_vec());
        }
        drop(cache);
        if data.len() > 2 && data[0] == b'x' {
            panic!("crash");
        }
    });
}
