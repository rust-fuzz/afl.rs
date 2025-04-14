fn main() {
    // This fuzz harness demonstrates the capabilities of CmpLog.
    // Simply run the fuzzer and it should find the crash immediately.
    afl::fuzz!(|data: &[u8]| {
        if data.len() < 29 {
            return;
        }
        if data[0] != b'A' {
            return;
        }
        if data[1] != b'B' {
            return;
        }
        if data[2] != b'C' {
            return;
        }
        if data[3] != b'D' {
            return;
        }

        if data[4..8] != 0x6969_4141_i32.to_le_bytes() {
            return;
        }

        if data[8..12] != *b"1234" || data[12..16] != *b"EFGH" {
            return;
        }

        let slice = &data[16..];
        let match_string = "Hello, world!";
        let compare_string = String::from_utf8(slice.to_vec()).unwrap_or_default();
        if compare_string != match_string {
            return;
        }

        panic!("BOOM");
    });
}
