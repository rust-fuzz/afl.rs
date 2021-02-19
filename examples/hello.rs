fn main() {
    afl::fuzz!(|data: &[u8]| {
        if data.get(0) == Some(&b'a') {
            panic!("Crash!")
        }
    });
}
