# Tutorial

For this tutorial, we are going to fuzz the URL parser [rust-url](https://github.com/servo/rust-url).

## Clone

Clone the repository and navigate inside:

```
git clone https://github.com/servo/rust-url
cd rust-url
```

## Dependencies

Modify `Cargo.toml` and add the following lines to the `[dependencies]` section:

```toml
afl = "0.1"
afl-plugin = "0.1"
```

The `afl-plugin` crate includes a compiler plugin that will be used to instrument the rust-url crate. The `afl` crate will be used to setup the necessary runtime afl expects during afl-fuzz. Also, it has a few other utilities.

## Instrumentation

Open up `src/lib.rs` and add these two lines before all other non-documentation/non-comment lines:

```
#![feature(plugin)]
#![plugin(afl_plugin)]
```

These will instrument.

## Driver

AFL requires an executable that will read from stdin. Create a new file `src/main.rs` and add the following contents:

```rust
extern crate afl;
extern crate url;

fn main() {
    afl::handle_string(|s| {
        let _ = url::Url::parse(s);
    })
}
```

## Input

AFL needs an input directory with test cases.

Make a new directory `in` and add a test:

```
mkdir in
echo "https://rust-lang.org" > in/basic
```

## Build

You'll need to enter the Docker environment to get the binary to compile correctly:

```
docker run -v $(pwd):/source -it corey/afl.rs sh
```

Run `cargo build` to compile it. It will create an executable at `target/debug/url`.

## Fuzz

```
afl-fuzz -i in -o out target/debug/url
```

## Exiting

You can exit `afl-fuzz` by pressing `ctrl-c` and you can exit the Docker environment by running the `exit` command.
