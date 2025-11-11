use cargo_afl_common::config;
use std::env;
use std::path::Path;

fn main() {
    let installing = home::cargo_home()
        .map(|path| Path::new(env!("CARGO_MANIFEST_DIR")).starts_with(path))
        .unwrap()
        || env::var("TESTING_INSTALL").is_ok();

    let building_on_docs_rs = env::var("DOCS_RS").is_ok();

    // smoelius: Build AFLplusplus only when installing and not building on docs.rs.
    if installing
        && !building_on_docs_rs
        && let Err(error) = config::config(&config::Args {
            build: true,
            force: true,
            plugins: cfg!(feature = "plugins"),
            ..Default::default()
        })
    {
        println!(
            "cargo:warn=Could not build AFLplusplus; it will need to be built manually with \
                 `cargo afl config --build`: {error}"
        );
    }
}
