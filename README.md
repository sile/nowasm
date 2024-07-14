nowasm
======

[![nowasm](https://img.shields.io/crates/v/nowasm.svg)](https://crates.io/crates/nowasm)
[![Documentation](https://docs.rs/nowasm/badge.svg)](https://docs.rs/nowasm)
[![Actions Status](https://github.com/sile/nowasm/workflows/CI/badge.svg)](https://github.com/sile/nowasm/actions)
![License](https://img.shields.io/crates/l/nowasm)


`nowasm` is a runtime library for [WebAssembly 1.0][wasm-core-1] that is implemented with no-std, no-unsafe and no-dependencies.
The goal is to provide a lightweight WebAssembly runtime that can be embedded wherever Rust is used, with a particular focus on Wasm-in-Wasm scenarios.


[wasm-core-1]: https://www.w3.org/TR/wasm-core-1/

TODO until v0.1.0
-----------------

- [ ] Add validation phase (TBD)
- [ ] Add doc comments
- [ ] Add more tests

Supported Extensions
--------------------

`nowasm` supports the following extensions that is necessary to run WebAssembly binaries build with the latest stable Rust compiler.
- [sign-extension]

[sign-extension]: https://github.com/WebAssembly/sign-extension-ops/blob/master/proposals/sign-extension-ops/Overview.md
