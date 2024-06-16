use clap::Parser;
use nowasm::ModuleSpec;
use orfail::{Failure, OrFail};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    wasm_path: PathBuf,
}

pub fn main() -> orfail::Result<()> {
    let args = Args::parse();
    let wasm_bytes = std::fs::read(&args.wasm_path).or_fail()?;
    let spec = ModuleSpec::inspect(&wasm_bytes)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail();
    println!("{spec:?}");
    Ok(())
}
