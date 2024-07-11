use clap::Parser;
use nowasm::{
    components::{Resulttype, Valtype},
    HostFunc, Module, Resolve, StdVectorFactory, Val,
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

    let mut instance = module
        .instantiate(Resolver)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;

    let func_args: Vec<_> = args.func_args.iter().copied().map(Val::I32).collect();
    let result = instance
        .invoke(&args.func_name, &func_args)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;
    println!("=> {:?}", result);

    Ok(())
}

struct Resolver;

impl Resolve for Resolver {
    type HostFunc = Print;

    fn resolve_func(
        &self,
        module: &str,
        name: &str,
        params: &[Valtype],
        result: Resulttype,
    ) -> Option<Self::HostFunc> {
        if module == "env"
            && name == "print"
            && params == [Valtype::I32, Valtype::I32]
            && result.len() == 0
        {
            Some(Print)
        } else {
            None
        }
    }
}

struct Print;

impl HostFunc for Print {
    fn invoke(&mut self, args: &[Val]) -> Option<Val> {
        // TODO: improve error handling (make it possible to return Err(_))
        // TODO: add module and store to args
        unsafe {
            let ptr = args[0].as_i32().unwrap() as *const u8;
            let len = args[1].as_i32().unwrap() as usize;
            let slice = std::slice::from_raw_parts(ptr, len);
            let string = std::str::from_utf8(slice).unwrap();
            print!("{string}");
        }
        None
    }
}
