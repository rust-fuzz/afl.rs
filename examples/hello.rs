fn main() {
    afl::fuzz!(|data: &[u8]| {
        if data.first() == Some(&b'a') {
            panic!("Crash!");
        }
    });
}
