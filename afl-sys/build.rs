extern crate gcc;

fn main() {
    // TODO: use better values here
    gcc::Config::new()
        .file("afl-2.52b/afl-fuzz.c")
        .define("BIN_PATH", Some("\"/tmp/bin/\""))  // TODO: does this value matter?
        .define("DOC_PATH", Some("\"/tmp/bin/\""))  // TODO: does this value matter?
        .define("VERSION", Some("\"2.52b\""))
        .define("main", Some("afl_fuzz_main"))  // Rename 'main' function
        .flag("-funroll-loops")
        .flag("-w")
        .include("afl-2.52b")
        .opt_level(3)
        .compile("libafl.a");
}
