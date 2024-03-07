#![allow(clippy::manual_assert)]

fn main() {
    afl::fuzz!(|n: u8| {
        if n == 100 && !cfg!(fuzzing) {
            panic!("Crash!");
        }
    });
}
