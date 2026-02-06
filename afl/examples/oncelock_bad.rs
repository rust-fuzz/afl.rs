// Demonstrates low stability in AFL++ persistent mode due to OnceLock.
// The OnceLock::get_or_init path diverges between iteration 1 (init runs)
// and subsequent iterations (init skipped), causing AFL's stability to drop.

use std::sync::OnceLock;

static CACHE: OnceLock<Vec<u8>> = OnceLock::new();

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let _cached = CACHE.get_or_init(|| data.to_vec());
        if data.len() > 2 && data[0] == b'x' {
            panic!("crash");
        }
    });
}
