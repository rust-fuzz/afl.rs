#[cfg(test)]
mod tests {
    use assert_cmd::Command;

    #[test]
    fn test_fuzz(){
        let mut cmd = Command::cargo_bin("fuzz").unwrap();
        cmd.env("RUSTFLAGS", "-C passes=sancov -Zsanitizer=address");
        let output = cmd.unwrap();
        assert!(output.status.success());
    }
}