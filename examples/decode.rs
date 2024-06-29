use clap::Parser;
use nowasm::{
    symbols::{Code, Data, Elem, Export, Global, Import, TableType, ValType},
    Counters, FixedSizeMutVector, FixedSizeMutVectors, FuncType, Instr, Locals, Module,
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
    let mut func_types = vec![FuncType::default(); counters.func_types];
    let mut imports = vec![Import::default(); counters.imports];
    let mut table_types = vec![TableType::default(); counters.table_types];
    let mut globals = vec![Global::default(); counters.globals];
    let mut exports = vec![Export::default(); counters.exports];
    let mut elems = vec![Elem::default(); counters.elems];
    let mut codes = vec![Code::default(); counters.codes];
    let mut datas = vec![Data::default(); counters.datas];

    let vectors = FixedSizeMutVectors {
        bytes: FixedSizeMutVector::new(&mut bytes),
        val_types: FixedSizeMutVector::new(&mut val_types),
        instrs: FixedSizeMutVector::new(&mut instrs),
        idxs: FixedSizeMutVector::new(&mut idxs),
        locals: FixedSizeMutVector::new(&mut locals),
        func_types: FixedSizeMutVector::new(&mut func_types),
        imports: FixedSizeMutVector::new(&mut imports),
        table_types: FixedSizeMutVector::new(&mut table_types),
        globals: FixedSizeMutVector::new(&mut globals),
        exports: FixedSizeMutVector::new(&mut exports),
        elems: FixedSizeMutVector::new(&mut elems),
        codes: FixedSizeMutVector::new(&mut codes),
        datas: FixedSizeMutVector::new(&mut datas),
    };

    let _module = Module::decode(&wasm_bytes, vectors)
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;

    println!("Decoded successfully!");

    Ok(())
}

fn inspect(wasm_bytes: &[u8]) -> orfail::Result<Counters> {
    let module = Module::decode(&wasm_bytes, Counters::new())
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;
    Ok(module.vectors().clone())
}
