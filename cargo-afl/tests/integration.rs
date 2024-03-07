use std::{path, process, thread, time};

fn target_dir_path() -> &'static path::Path {
    if path::Path::new("../target/debug/cargo-afl").exists() {
        path::Path::new("../target/debug/")
    } else if path::Path::new("target/debug/cargo-afl").exists() {
        path::Path::new("target/debug/")
    } else {
        panic!("Could not find cargo-afl binary");
    }
}

fn cargo_afl_path() -> path::PathBuf {
    target_dir_path().join("cargo-afl")
}

fn examples_path(name: &str) -> path::PathBuf {
    target_dir_path().join("examples").join(name)
}

fn input_path() -> path::PathBuf {
    path::Path::new(env!("CARGO_MANIFEST_DIR")).join("input")
}

#[test]
fn integration() {
    let temp_dir = tempfile::TempDir::new().expect("Could not create temporary directory");
    let temp_dir_path = temp_dir.path();
    let mut child = process::Command::new(cargo_afl_path())
        .arg("afl")
        .arg("fuzz")
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .arg("-i")
        .arg(input_path())
        .arg("-o")
        .arg(temp_dir_path)
        .arg(examples_path("hello"))
        .env("AFL_NO_UI", "1")
        .spawn()
        .expect("Could not run cargo afl fuzz");
    thread::sleep(time::Duration::from_secs(10));
    for _ in 0..5 {
        thread::sleep(time::Duration::from_secs(1));
        child.kill().unwrap_or_default();
    }
    assert!(temp_dir_path.join("default").join("fuzzer_stats").is_file());
}

#[test]
fn integration_cfg() {
    let temp_dir = tempfile::TempDir::new().expect("Could not create temporary directory");
    let temp_dir_path = temp_dir.path();

    assert_cmd::Command::new(cargo_afl_path())
        .arg("afl")
        .arg("build")
        .arg("--example")
        .arg("cfg")
        .env("AFL_NO_CFG_FUZZING", "1")
        .arg("-vv")
        .arg("--manifest-path")
        .arg("../afl/Cargo.toml")
        .assert()
        .success();

    let mut child = process::Command::new(cargo_afl_path())
        .arg("afl")
        .arg("fuzz")
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .arg("-i")
        .arg(input_path())
        .arg("-o")
        .arg(temp_dir_path)
        .arg(examples_path("cfg"))
        .env("AFL_NO_UI", "1")
        .spawn()
        .expect("Could not run cargo afl fuzz");
    thread::sleep(time::Duration::from_secs(5));
    for _ in 0..5 {
        thread::sleep(time::Duration::from_secs(1));
        child.kill().unwrap_or_default();
    }
    assert!(temp_dir_path.join("default").join("fuzzer_stats").is_file());
    let crashes = std::fs::read_dir(temp_dir_path.join("default").join("crashes"))
        .unwrap()
        .count()
        - 1;
    assert_eq!(crashes, 1);
}
