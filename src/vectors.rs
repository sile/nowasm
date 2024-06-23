use crate::{
    decode::Decode,
    instructions::Instr,
    reader::Reader,
    symbols::{Code, Data, Elem, Export, FuncType, Global, Import, Locals, TableType, ValType},
    DecodeError,
};
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct VectorSlice<T> {
    offset: usize,
    len: usize,
    _item: PhantomData<T>,
}

impl<T> VectorSlice<T> {
    pub fn len(self) -> usize {
        self.len
    }

    pub fn is_empty(self) -> bool {
        self.len == 0
    }
}

impl<T> Default for VectorSlice<T> {
    fn default() -> Self {
        Self {
            offset: 0,
            len: 0,
            _item: PhantomData,
        }
    }
}

impl<T: VectorItem> Decode for VectorSlice<T> {
    fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError> {
        let offset = T::append(vectors, &[])?;
        let len = reader.read_usize()?;
        for _ in 0..len {
            let item = T::decode(reader, vectors)?;
            T::append(vectors, &[item])?;
        }
        Ok(Self {
            offset,
            len,
            _item: PhantomData,
        })
    }
}

pub trait VectorItem: Decode {
    fn append<V: Vectors>(vectors: &mut V, items: &[Self]) -> Result<usize, DecodeError>;
    // TODO: Add decode()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorKind {
    Bytes,
    ValTypes,
    Instrs,
    Idxs,
    Locals,
    FuncTypes,
    Imports,
    TableTypes,
    Globals,
    Exports,
    Elems,
    Codes,
    Datas,
}

pub trait Vectors {
    fn bytes_offset(&self) -> usize; // TODO: bytes(&self) -> &[u8]
    fn bytes_append(&mut self, bytes: &[u8]) -> bool;

    fn val_types_offset(&self) -> usize;
    fn val_types_push(&mut self, val_type: ValType) -> bool;

    fn instrs_offset(&self) -> usize;
    fn instrs_push(&mut self, instr: Instr) -> bool;

    fn idxs_offset(&self) -> usize;
    fn idxs_push(&mut self, idx: u32) -> bool;

    fn idxs_append<T: Into<u32>>(&mut self, idxs: &[T]) -> bool;
    fn idxs(&self) -> &[u32];

    fn locals_offset(&self) -> usize;
    fn locals_push(&mut self, locals: Locals) -> bool;

    fn func_types(&self) -> &[FuncType];
    fn func_types_append(&mut self, items: &[FuncType]) -> bool;

    fn imports(&self) -> &[Import];
    fn imports_append(&mut self, items: &[Import]) -> bool;

    fn table_types(&self) -> &[TableType];
    fn table_types_append(&mut self, items: &[TableType]) -> bool;

    fn globals(&self) -> &[Global];
    fn globals_append(&mut self, items: &[Global]) -> bool;

    fn exports(&self) -> &[Export];
    fn exports_append(&mut self, items: &[Export]) -> bool;

    fn elems(&self) -> &[Elem];
    fn elems_append(&mut self, items: &[Elem]) -> bool;

    fn codes(&self) -> &[Code];
    fn codes_append(&mut self, items: &[Code]) -> bool;

    fn datas(&self) -> &[Data];
    fn datas_append(&mut self, items: &[Data]) -> bool;
}

#[derive(Debug, Default)]
pub struct Counters {
    pub bytes: usize,
    pub val_types: usize,
    pub instrs: usize,
    pub idxs: usize,
    pub locals: usize,
    pub func_types: usize,
    pub imports: usize,
    pub table_types: usize,
    pub globals: usize,
    pub exports: usize,
    pub elems: usize,
    pub codes: usize,
    pub datas: usize,
}

impl Counters {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Vectors for Counters {
    fn bytes_offset(&self) -> usize {
        self.bytes
    }

    fn bytes_append(&mut self, bytes: &[u8]) -> bool {
        self.bytes += bytes.len();
        true
    }

    fn val_types_offset(&self) -> usize {
        self.val_types
    }

    fn val_types_push(&mut self, _val_type: ValType) -> bool {
        self.val_types += 1;
        true
    }

    fn instrs_offset(&self) -> usize {
        self.instrs
    }

    fn instrs_push(&mut self, _instr: Instr) -> bool {
        self.instrs += 1;
        true
    }

    fn idxs_offset(&self) -> usize {
        self.idxs
    }

    fn idxs_push(&mut self, _idx: u32) -> bool {
        self.idxs += 1;
        true
    }

    fn locals_offset(&self) -> usize {
        self.locals
    }

    fn locals_push(&mut self, _locals: Locals) -> bool {
        self.locals += 1;
        true
    }

    fn func_types(&self) -> &[FuncType] {
        &[]
    }

    fn func_types_append(&mut self, items: &[FuncType]) -> bool {
        self.func_types += items.len();
        true
    }

    fn imports(&self) -> &[Import] {
        &[]
    }

    fn imports_append(&mut self, items: &[Import]) -> bool {
        self.imports += items.len();
        true
    }

    fn idxs_append<T: Into<u32>>(&mut self, idxs: &[T]) -> bool {
        self.idxs += idxs.len();
        true
    }

    fn idxs(&self) -> &[u32] {
        &[]
    }

    fn table_types(&self) -> &[TableType] {
        &[]
    }

    fn table_types_append(&mut self, items: &[TableType]) -> bool {
        self.table_types += items.len();
        true
    }

    fn globals(&self) -> &[Global] {
        &[]
    }

    fn globals_append(&mut self, items: &[Global]) -> bool {
        self.globals += items.len();
        true
    }

    fn exports(&self) -> &[Export] {
        &[]
    }

    fn exports_append(&mut self, items: &[Export]) -> bool {
        self.exports += items.len();
        true
    }

    fn elems(&self) -> &[Elem] {
        &[]
    }

    fn elems_append(&mut self, items: &[Elem]) -> bool {
        self.elems += items.len();
        true
    }

    fn codes(&self) -> &[Code] {
        &[]
    }

    fn codes_append(&mut self, items: &[Code]) -> bool {
        self.codes += items.len();
        true
    }

    fn datas(&self) -> &[Data] {
        &[]
    }

    fn datas_append(&mut self, items: &[Data]) -> bool {
        self.datas += items.len();
        true
    }
}
