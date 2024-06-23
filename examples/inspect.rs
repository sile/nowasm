use clap::Parser;
use nowasm::{Counters, Module};
use orfail::{Failure, OrFail};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    wasm_path: PathBuf,
}

pub fn main() -> orfail::Result<()> {
    let args = Args::parse();
    let wasm_bytes = std::fs::read(&args.wasm_path).or_fail()?;
    let module = Module::decode(&wasm_bytes, Counters::new())
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;
    println!("{:?}", module.vectors());
    Ok(())
}
