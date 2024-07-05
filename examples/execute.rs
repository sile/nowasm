use clap::Parser;
use nowasm::{
    execution::{Frame, ImportObject, ModuleInstance, Stacks, Store, Value},
    symbols::{Code, Data, Elem, Export, Global, GlobalIdx, Import, TableType, ValType},
    FuncType, Instr, Locals, Module, Vectors,
};
use orfail::{Failure, OrFail};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    wasm_path: PathBuf,
    func_name: String,
    func_args: Vec<i32>,
}

pub fn main() -> orfail::Result<()> {
    let args = Args::parse();
    let wasm_bytes = std::fs::read(&args.wasm_path).or_fail()?;

    let module = Module::decode(&wasm_bytes, StdVectors::default())
        .map_err(|e| Failure::new(format!("{e:?}")))
        .or_fail()?;

    let mut instance = ModuleInstance::new(
        module,
        ExampleStore::default(),
        ExampleStacks::default(),
        ExampleImportObject::default(),
    )
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

#[derive(Debug, Default)]
pub struct ExampleStore {
    globals: Vec<Value>,
}

impl Store for ExampleStore {
    fn push_global(&mut self, value: Value) {
        self.globals.push(value);
    }

    fn set_global(&mut self, i: GlobalIdx, value: Value) {
        self.globals[i.get() as usize] = value;
    }

    fn get_global(&self, i: GlobalIdx) -> Value {
        self.globals[i.get() as usize]
    }
}

#[derive(Debug, Default)]
pub struct ExampleStacks {
    frames: Vec<ExampleFrame>,
}

impl Stacks for ExampleStacks {
    fn push_frame(&mut self, locals: usize) {
        self.frames.push(ExampleFrame {
            locals: vec![Value::I32(0); locals],
        });
    }

    fn pop_frame(&mut self) {
        self.frames.pop();
    }

    fn current_frame(&mut self) -> Frame {
        let Some(last) = self.frames.last_mut() else {
            return Frame { locals: &mut [] };
        };
        Frame {
            locals: &mut last.locals,
        }
    }
}

#[derive(Debug, Clone)]
struct ExampleFrame {
    locals: Vec<Value>,
}

#[derive(Debug, Default)]
pub struct ExampleImportObject {}

impl ImportObject for ExampleImportObject {}

#[derive(Debug, Default)]
pub struct StdVectors {
    pub bytes: Vec<u8>,
    pub val_types: Vec<ValType>,
    pub instrs: Vec<Instr>,
    pub idxs: Vec<u32>,
    pub locals: Vec<Locals>,
    pub func_types: Vec<FuncType>,
    pub imports: Vec<Import>,
    pub table_types: Vec<TableType>,
    pub globals: Vec<Global>,
    pub exports: Vec<Export>,
    pub elems: Vec<Elem>,
    pub codes: Vec<Code>,
    pub datas: Vec<Data>,
}

impl Vectors for StdVectors {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn bytes_append(&mut self, bytes: &[u8]) -> bool {
        self.bytes.extend_from_slice(bytes);
        true
    }

    fn val_types(&self) -> &[ValType] {
        &self.val_types
    }

    fn val_types_append(&mut self, items: &[ValType]) -> bool {
        self.val_types.extend_from_slice(items);
        true
    }

    fn instrs(&self) -> &[Instr] {
        &self.instrs
    }

    fn instrs_append(&mut self, items: &[Instr]) -> bool {
        self.instrs.extend_from_slice(items);
        true
    }

    fn idxs(&self) -> &[u32] {
        &self.idxs
    }

    fn idxs_append<T: Copy + Into<u32>>(&mut self, idxs: &[T]) -> bool {
        self.idxs.extend(idxs.iter().map(|&x| x.into()));
        true
    }

    fn locals(&self) -> &[Locals] {
        &self.locals
    }

    fn locals_append(&mut self, items: &[Locals]) -> bool {
        self.locals.extend_from_slice(items);
        true
    }

    fn func_types(&self) -> &[FuncType] {
        &self.func_types
    }

    fn func_types_append(&mut self, items: &[FuncType]) -> bool {
        self.func_types.extend_from_slice(items);
        true
    }

    fn imports(&self) -> &[Import] {
        &self.imports
    }

    fn imports_append(&mut self, items: &[Import]) -> bool {
        self.imports.extend_from_slice(items);
        true
    }

    fn table_types(&self) -> &[TableType] {
        &self.table_types
    }

    fn table_types_append(&mut self, items: &[TableType]) -> bool {
        self.table_types.extend_from_slice(items);
        true
    }

    fn globals(&self) -> &[Global] {
        &self.globals
    }

    fn globals_append(&mut self, items: &[Global]) -> bool {
        self.globals.extend_from_slice(items);
        true
    }

    fn exports(&self) -> &[Export] {
        &self.exports
    }

    fn exports_append(&mut self, items: &[Export]) -> bool {
        self.exports.extend_from_slice(items);
        true
    }

    fn elems(&self) -> &[Elem] {
        &self.elems
    }

    fn elems_append(&mut self, items: &[Elem]) -> bool {
        self.elems.extend_from_slice(items);
        true
    }

    fn codes(&self) -> &[Code] {
        &self.codes
    }

    fn codes_append(&mut self, items: &[Code]) -> bool {
        self.codes.extend_from_slice(items);
        true
    }

    fn datas(&self) -> &[Data] {
        &self.datas
    }

    fn datas_append(&mut self, items: &[Data]) -> bool {
        self.datas.extend_from_slice(items);
        true
    }
}
