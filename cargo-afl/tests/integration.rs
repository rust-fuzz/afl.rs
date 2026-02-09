use cargo_afl_common as common;
use std::{
    io::Write,
    path,
    process::{self, ExitStatus},
};

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
    if !common::plugins_installed().unwrap_or_default() {
        #[allow(clippy::explicit_write)]
        writeln!(
            std::io::stderr(),
            "Skipping `integration_maze` test as plugins are not installed"
        )
        .unwrap();
        return;
    }

    let temp_dir = tempfile::TempDir::new().expect("Could not create temporary directory");
    let temp_dir_path = temp_dir.path();

    for _i in 0..3 {
        let _: ExitStatus = process::Command::new(cargo_afl_path())
            .arg("afl")
            .arg("fuzz")
            .arg("-i")
            .arg(input_path())
            .arg("-o")
            .arg(temp_dir_path)
            .args(["-V", "15"]) // 15 seconds
            .arg(examples_path("maze"))
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
        if crashes >= 1 {
            return;
        }
    }

    unreachable!();
}

#[test]
fn integration_fuzz_with_reset() {
    // Run without reset (expect low stability)
    let dir_no_reset = fuzz_example_with_envs("reset_demo", 15, &[]);

    // Run with reset (expect high stability)
    let dir_with_reset = fuzz_example_with_envs("reset_demo", 15, &[("USE_RESET", "1")]);

    let stability_no_reset = parse_stability(dir_no_reset.path());
    let stability_with_reset = parse_stability(dir_with_reset.path());

    assert!(
        stability_no_reset < 90.0,
        "Stability without reset ({stability_no_reset}%) should be below 90%"
    );
    assert!(
        stability_with_reset > 90.0,
        "Stability with reset ({stability_with_reset}%) should be above 90%"
    );
}

fn fuzz_example(name: &str, should_crash: bool) {
    let temp_dir = fuzz_example_with_envs(name, 5, &[("AFL_BENCH_UNTIL_CRASH", "1")]);
    let temp_dir_path = temp_dir.path();
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

fn fuzz_example_with_envs(
    name: &str,
    timeout_secs: u32,
    envs: &[(&str, &str)],
) -> tempfile::TempDir {
    let temp_dir = tempfile::TempDir::new().expect("Could not create temporary directory");
    let _: ExitStatus = process::Command::new(cargo_afl_path())
        .arg("afl")
        .arg("fuzz")
        .arg("-i")
        .arg(input_path())
        .arg("-o")
        .arg(temp_dir.path())
        .args(["-V", &timeout_secs.to_string()])
        .arg(examples_path(name))
        .env("AFL_NO_CRASH_README", "1")
        .env("AFL_NO_UI", "1")
        .envs(envs.iter().copied())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .status()
        .expect("Could not run cargo afl fuzz");
    temp_dir
}

fn parse_stability(output_dir: &path::Path) -> f64 {
    let stats_path = output_dir.join("default").join("fuzzer_stats");
    let contents = std::fs::read_to_string(&stats_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", stats_path.display()));
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("stability") {
            let value = value
                .trim()
                .trim_start_matches(':')
                .trim()
                .trim_end_matches('%');
            return value
                .parse()
                .unwrap_or_else(|e| panic!("Failed to parse stability value '{value}': {e}"));
        }
    }
    panic!("No stability line found in {}", stats_path.display());
}
