use std::{
    io::Write,
    path,
    process::{self, ExitStatus},
};

#[allow(dead_code)]
#[path = "../src/common.rs"]
mod common;

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
    fuzz_example("hello", true);
}

#[test]
fn integration_cfg() {
    for cfg_fuzzing in [false, true] {
        assert_cmd::Command::new(cargo_afl_path())
            .arg("afl")
            .arg("build")
            .arg("--example")
            .arg("cfg")
            .arg("--manifest-path")
            .arg("../afl/Cargo.toml")
            .envs(if cfg_fuzzing {
                vec![]
            } else {
                vec![("AFL_NO_CFG_FUZZING", "1")]
            })
            .assert()
            .success();

        // Assert that if cfg_fuzzing is set, there are no crashes
        // And if it is not set, there is at least one crash
        fuzz_example("cfg", !cfg_fuzzing);
    }
}

#[test]
fn integration_maze() {
    if !common::plugins_available().unwrap_or_default() {
        #[allow(clippy::explicit_write)]
        writeln!(
            std::io::stderr(),
            "Skipping `integration_maze` test as plugins are unavailable"
        )
        .unwrap();
        return;
    }


    fuzz_example("maze", true);
}

fn fuzz_example(name: &str, should_crash: bool) {
    let temp_dir = tempfile::TempDir::new().expect("Could not create temporary directory");
    let temp_dir_path = temp_dir.path();
    let _: ExitStatus = process::Command::new(cargo_afl_path())
        .arg("afl")
        .arg("fuzz")
        .arg("-i")
        .arg(input_path())
        .arg("-o")
        .arg(temp_dir_path)
        .args(["-V", "10"]) // 5 seconds
        .arg(examples_path(name))
        .env("AFL_BENCH_UNTIL_CRASH", "1")
        .env("AFL_NO_CRASH_README", "1")
        .env("AFL_NO_UI", "1")
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .status()
        .expect("Could not run cargo afl fuzz");
    assert!(temp_dir_path.join("default").join("fuzzer_stats").is_file());
    let crashes = std::fs::read_dir(temp_dir_path.join("default").join("crashes"))
        .unwrap()
        .count();
    if should_crash {
        assert!(crashes >= 1);
    } else {
        assert_eq!(0, crashes);
    }
}
