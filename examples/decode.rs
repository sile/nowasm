use clap::Parser;
use nowasm::{
    symbols::ValType, Counters, FixedSizeMutVector, FixedSizeMutVectors, Instr, Locals, Module,
};
use orfail::{Failure, OrFail};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    wasm_path: PathBuf,
}

pub fn main() -> orfail::Result<()> {
    let args = Args::parse();
    let wasm_bytes = std::fs::read(&args.wasm_path).or_fail()?;

    let counters = inspect(&wasm_bytes).or_fail()?;

    let mut bytes = vec![0; counters.bytes];
    let mut val_types = vec![ValType::I32; counters.val_types];
    let mut instrs = vec![Instr::Nop; counters.instrs];
    let mut idxs = vec![0; counters.idxs];
    let mut locals = vec![
        Locals {
            n: 0,
            t: ValType::I32
        };
        counters.locals
    ];
    let vectors = FixedSizeMutVectors {
        bytes: FixedSizeMutVector::new(&mut bytes),
        val_types: FixedSizeMutVector::new(&mut val_types),
        instrs: FixedSizeMutVector::new(&mut instrs),
        idxs: FixedSizeMutVector::new(&mut idxs),
        locals: FixedSizeMutVector::new(&mut locals),
        // pub func_types: FixedSizeMutVector<'a, FuncType>,
        // pub imports: FixedSizeMutVector<'a, Import>,
        // pub table_types: FixedSizeMutVector<'a, TableType>,
        // pub globals: FixedSizeMutVector<'a, Global>,
        // pub exports: FixedSizeMutVector<'a, Export>,
        // pub elems: FixedSizeMutVector<'a, Elem>,
        // pub codes: FixedSizeMutVector<'a, Code>,
        // pub datas: FixedSizeMutVector<'a, Data>,
    };

    let module = Module::decode(&wasm_bytes, vectors)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;

    // TODO: Print information about the module

    Ok(())
}

fn inspect(wasm_bytes: &[u8]) -> orfail::Result<Counters> {
    let module = Module::decode(&wasm_bytes, Counters::new())
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;
    Ok(module.vectors().clone())
}
