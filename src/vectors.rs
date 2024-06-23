use crate::{
    instructions::Instr,
    reader::Reader,
    symbols::{Code, Data, Elem, Export, FuncType, Global, Import, Locals, TableType, ValType},
    DecodeError,
};
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct VectorSlice<T> {
    pub offset: usize, // TODO: priv
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

// TODO: move
impl<T: VectorItem> VectorSlice<T> {
    pub fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError> {
        let offset = T::append(&[], vectors)?;
        let len = reader.read_usize()?;
        for _ in 0..len {
            let item = T::decode(reader, vectors)?;
            T::append(&[item], vectors)?;
        }
        Ok(Self {
            offset,
            len,
            _item: PhantomData,
        })
    }
}

pub trait VectorItem: Sized {
    fn append<V: Vectors>(items: &[Self], vectors: &mut V) -> Result<usize, DecodeError>;
    fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError>;
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
    fn bytes(&self) -> &[u8];
    fn bytes_append(&mut self, bytes: &[u8]) -> bool;

    fn val_types(&self) -> &[ValType];
    fn val_types_append(&mut self, items: &[ValType]) -> bool;

    fn instrs(&self) -> &[Instr];
    fn instrs_append(&mut self, items: &[Instr]) -> bool;

    fn idxs(&self) -> &[u32];
    fn idxs_append<T: Into<u32>>(&mut self, idxs: &[T]) -> bool;

    // TODO: name
    fn locals(&self) -> &[Locals];
    fn locals_append(&mut self, items: &[Locals]) -> bool;

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
    fn bytes(&self) -> &[u8] {
        &[]
    }

    fn bytes_append(&mut self, bytes: &[u8]) -> bool {
        self.bytes += bytes.len();
        true
    }

    fn val_types(&self) -> &[ValType] {
        &[]
    }

    fn val_types_append(&mut self, items: &[ValType]) -> bool {
        self.val_types += items.len();
        true
    }

    fn instrs(&self) -> &[Instr] {
        &[]
    }

    fn instrs_append(&mut self, items: &[Instr]) -> bool {
        self.instrs += items.len();
        true
    }

    fn locals(&self) -> &[Locals] {
        &[]
    }

    fn locals_append(&mut self, items: &[Locals]) -> bool {
        self.locals += items.len();
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
