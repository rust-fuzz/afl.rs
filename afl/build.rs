use std::env;
use std::path::Path;

fn main() {
    let building_in_cargo_home = home::cargo_home()
        .map(|path| Path::new(env!("CARGO_MANIFEST_DIR")).starts_with(path))
        .unwrap()
        || env::var("TESTING_BUILD").is_ok();

    let cfg_fuzzing = env::var("CARGO_CFG_FUZZING").is_ok();
    let cfg_no_fuzzing = env::var("CARGO_CFG_NO_FUZZING").is_ok();

    if building_in_cargo_home && !cfg_fuzzing && !cfg_no_fuzzing {
        println!("cargo:warning=You appear to be building `afl` not under `cargo-afl`.");
        println!("cargo:warning=Perhaps you used `cargo build` instead of `cargo afl build`?");
    }
}
