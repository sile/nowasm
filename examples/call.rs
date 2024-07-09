use clap::Parser;
use nowasm::{
    execute::{ModuleInstanceOptions, Value},
    Module, StdVector, StdVectorFactory,
};
use orfail::{Failure, OrFail};
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug, Parser)]
struct Args {
    wasm_path: PathBuf,
    func_name: String,
    func_args: Vec<i32>,
}

pub fn main() -> orfail::Result<()> {
    let args = Args::parse();
    let wasm_bytes = std::fs::read(&args.wasm_path).or_fail()?;

    let module = Module::<StdVectorFactory>::decode(&wasm_bytes)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;

    let options = ModuleInstanceOptions {
        mem: Some(StdVector::new(vec![0; 1024 * 1024])),
        ..Default::default()
    };
    let mut instance = module
        .instantiate(options)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;

    let func_args: Vec<_> = args.func_args.iter().copied().map(Value::I32).collect();
    let result = instance
        .invoke(&args.func_name, &func_args)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;
    println!("=> {:?}", result);

    Ok(())
}
