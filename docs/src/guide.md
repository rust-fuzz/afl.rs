# Guide

### Deferred init

If your program has a slow set-up phase that does not depend on the input data,
you can set `AFL_DEFER_FORKSRV=1` for a substantial speed-up, provided that you
insert a call to `afl::init()` after the set-up and before any
dependence on input. There are various other caveats, described in the section
"Bonus feature: deferred instrumentation" in `llvm_mode/README.llvm`
distributed with afl. See also [`examples/deferred-init.rs`][example-defer] in
this repository.

### Conditional compilation

afl instrumentation adds some run-time overhead, so it's a good candidate for
[conditional compilation][], perhaps through a [Cargo feature][]:

```toml
# You may need to add `optional = true` to the above dependencies.
[features]
afl = ["afl-plugin", "afl"]
```

```rust
// Active only with `cargo [...] --feature afl`
#![cfg_attr(feature = "afl", feature(plugin))]
#![cfg_attr(feature = "afl", plugin(afl_plugin))]
```


### AFL configuration

See the afl documentation for other configuration variables. Some of these are
set at compile time in `config.h`. For the most part they only affect
`afl-fuzz` itself, and will work fine with this library. However, if you change
`SHM_ENV_VAR`, `MAP_SIZE`, or `FORKSRV_FD`, you should update this library's
`src/config.h` to match.
