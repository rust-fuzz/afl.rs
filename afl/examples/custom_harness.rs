#![allow(clippy::manual_assert)]

use std::io::Read;

fn main() {
    afl::afl_init!();

    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap();

    if input.len() > 3 && input[0] == b'X' {
        panic!("found a bug!");
    }
}
