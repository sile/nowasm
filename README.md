nowasm
======

[![nowasm](https://img.shields.io/crates/v/nowasm.svg)](https://crates.io/crates/nowasm)
[![Documentation](https://docs.rs/nowasm/badge.svg)](https://docs.rs/nowasm)
[![Actions Status](https://github.com/sile/nowasm/workflows/CI/badge.svg)](https://github.com/sile/nowasm/actions)
![License](https://img.shields.io/crates/l/nowasm)


`nowasm` is a runtime library for [WebAssembly 1.0][wasm-core-1] that is implemented with no-std, no-unsafe and no-dependencies.

The goal is to provide a lightweight WebAssembly runtime that can be embedded wherever Rust is used, with a particular focus on Wasm-in-Wasm scenarios.

[wasm-core-1]: https://www.w3.org/TR/wasm-core-1/

TODO list until v0.1.0
----------------------

- [ ] Add validation phase (TBD)
- [ ] Add doc comments
- [ ] Add more tests

Supported Extensions
--------------------

`nowasm` supports the following extensions necessary to run WebAssembly binaries built with the latest stable Rust compiler.
- [sign-extension]

[sign-extension]: https://github.com/WebAssembly/sign-extension-ops/blob/master/proposals/sign-extension-ops/Overview.md

Examples
--------

Please execute the command `$ cargo build --target wasm32-unknown-unknown --example hello` to build the following"Hello World!" printing code ([examples/wasm/hello.rs](examples/wasm/hello.rs)) into a WebAssembly binary:

```rust
extern "C" {
    fn print(s: *const u8, len: i32);
}

#[no_mangle]
pub fn hello() {
    let msg = "Hello, World!\n";
    unsafe {
        print(msg.as_ptr(), msg.len() as i32);
    }
}
```


Then, you can call the `hello()` function via the following command:
```console
$ cargo run --example call_hello
Hello, World!
```

The code of [examples/call_hello.rs](examples/call_hello.rs) is as follows:
```rust
use nowasm::{Env, HostFunc, Module, Resolve, StdVectorFactory, Val};

pub fn main() {
    let wasm_bytes = include_bytes!("../target/wasm32-unknown-unknown/debug/examples/hello.wasm");

    let module = Module::<StdVectorFactory>::decode(wasm_bytes).expect("Failed to decode module");

    let mut instance = module
        .instantiate(Resolver)
        .expect("Failed to instantiate module");

    instance
        .invoke("hello", &[])
        .expect("Failed to invoke function");
}

struct Resolver;

impl Resolve for Resolver {
    type HostFunc = Print;

    fn resolve_func(&self, module: &str, name: &str) -> Option<Self::HostFunc> {
        assert_eq!(module, "env");
        assert_eq!(name, "print");
        Some(Print)
    }
}

struct Print;

impl HostFunc for Print {
    fn invoke(&mut self, args: &[Val], env: &mut Env) -> Option<Val> {
        let ptr = args[0].as_i32().unwrap() as usize;
        let len = args[1].as_i32().unwrap() as usize;
        let string = std::str::from_utf8(&env.mem[ptr..ptr + len]).expect("Invalid utf8");
        print!("{string}");
        None
    }
}
```
