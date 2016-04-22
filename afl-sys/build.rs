extern crate gcc;

fn main() {
    // TODO: use better values here
    gcc::Config::new()
        .file("afl-2.10b/afl-fuzz.c")
        .define("BIN_PATH", Some("\"/tmp/bin/\""))  // TODO: does this value matter?
        .define("DOC_PATH", Some("\"/tmp/bin/\""))  // TODO: does this value matter?
        .define("VERSION", Some("\"2.10b\""))
        .define("main", Some("afl_fuzz_main"))  // Rename 'main' function
        .flag("-funroll-loops")
        .include("afl-2.10b")
        .opt_level(3)
        .compile("libafl.a");
}
