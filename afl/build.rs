use std::env;
use std::path::Path;

fn main() {
    let building_in_cargo_home = home::cargo_home()
        .map(|path| Path::new(env!("CARGO_MANIFEST_DIR")).starts_with(path))
        .unwrap()
        || env::var("TESTING_BUILD").is_ok();

    let cfg_fuzzing = env::var("CARGO_CFG_FUZZING").is_ok();
    let no_cfg_fuzzing = !env::var("AFL_NO_CFG_FUZZING")
        .unwrap_or_default()
        .is_empty();

    // afl-fuzz is sensitive to AFL_ env variables. Let's remove this particular one - it did it's job
    env::remove_var("AFL_NO_CFG_FUZZING");

    if building_in_cargo_home && !cfg_fuzzing && !no_cfg_fuzzing {
        println!("cargo:warning=You appear to be building `afl` not under `cargo-afl`.");
        println!("cargo:warning=Perhaps you used `cargo build` instead of `cargo afl build`?");
    }
}
