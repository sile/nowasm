use crate::{
    instructions::Instr,
    reader::Reader,
    symbols::{Code, Data, Elem, Export, FuncType, Global, Import, Locals, TableType, ValType},
    DecodeError,
};
use core::{marker::PhantomData, slice::SliceIndex};

pub trait Allocator {
    type Vector<T: Clone>: Vector<T>;

    fn allocate_vector<T: Clone>() -> Self::Vector<T>;
}

pub trait Vector<T: Clone>: AsRef<[T]> + AsMut<[T]> {
    fn push(&mut self, item: T);
}

#[derive(Debug)]
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

impl<T> Clone for VectorSlice<T> {
    fn clone(&self) -> Self {
        Self {
            offset: self.offset,
            len: self.len,
            _item: PhantomData,
        }
    }
}

impl<T> Copy for VectorSlice<T> {}

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

    pub fn get(self, index: usize, vectors: &impl Vectors) -> Option<T> {
        if index >= self.len {
            return None;
        }
        T::get(self.offset + index, vectors)
    }

    pub fn iter<'a, V: Vectors>(self, vectors: &'a V) -> Iter<'a, T, V> {
        Iter::new(self, vectors)
    }
}

#[derive(Debug)]
pub struct Iter<'a, T, V> {
    vectors: &'a V,
    slice: VectorSlice<T>,
    i: usize,
}

impl<'a, T, V> Iter<'a, T, V>
where
    T: VectorItem,
    V: Vectors,
{
    fn new(slice: VectorSlice<T>, vectors: &'a V) -> Self {
        Self {
            slice,
            vectors,
            i: 0,
        }
    }
}

impl<'a, T, V> Iterator for Iter<'a, T, V>
where
    T: VectorItem,
    V: Vectors,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(item) = self.slice.get(self.i, self.vectors) else {
            return None;
        };
        self.i += 1;
        Some(item)
    }
}

pub trait VectorItem: Sized {
    fn append<V: Vectors>(items: &[Self], vectors: &mut V) -> Result<usize, DecodeError>;
    fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError>;
    fn get(index: usize, vectors: &impl Vectors) -> Option<Self>;
}

// TODO: rename
pub trait DecodeVector: Sized + Clone {
    fn decode_item(reader: &mut Reader) -> Result<Self, DecodeError>;
    fn decode_vector<A: Allocator>(reader: &mut Reader) -> Result<A::Vector<Self>, DecodeError> {
        let len = reader.read_usize()?;
        let mut vs = A::allocate_vector();
        for _ in 0..len {
            let item = Self::decode_item(reader)?;
            vs.push(item);
        }
        Ok(vs)
    }
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
    fn idxs_append<T: Copy + Into<u32>>(&mut self, idxs: &[T]) -> bool;

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

#[derive(Debug)]
pub struct FixedSizeMutVector<'a, T> {
    items: &'a mut [T],
    len: usize,
}

impl<'a, T: Copy> FixedSizeMutVector<'a, T> {
    pub fn new(items: &'a mut [T]) -> Self {
        Self { items, len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get<I: SliceIndex<[T]>>(&self, index: I) -> Option<&I::Output> {
        self.items.get(index)
    }

    pub fn append(&mut self, items: &[T]) -> bool {
        if self.len + items.len() <= self.items.len() {
            self.items[self.len..self.len + items.len()].copy_from_slice(items);
            self.len += items.len();
            true
        } else {
            false
        }
    }

    // TODO
    pub fn append_iter(&mut self, items: impl Iterator<Item = T>) -> bool {
        for item in items {
            if !self.append(&[item]) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug)]
pub struct FixedSizeMutVectors<'a> {
    pub bytes: FixedSizeMutVector<'a, u8>,
    pub val_types: FixedSizeMutVector<'a, ValType>,
    pub instrs: FixedSizeMutVector<'a, Instr>,
    pub idxs: FixedSizeMutVector<'a, u32>,
    pub locals: FixedSizeMutVector<'a, Locals>,
    pub func_types: FixedSizeMutVector<'a, FuncType>,
    pub imports: FixedSizeMutVector<'a, Import>,
    pub table_types: FixedSizeMutVector<'a, TableType>,
    pub globals: FixedSizeMutVector<'a, Global>,
    pub exports: FixedSizeMutVector<'a, Export>,
    pub elems: FixedSizeMutVector<'a, Elem>,
    pub codes: FixedSizeMutVector<'a, Code>,
    pub datas: FixedSizeMutVector<'a, Data>,
}

impl<'a> Vectors for FixedSizeMutVectors<'a> {
    fn bytes(&self) -> &[u8] {
        &self.bytes.items[..self.bytes.len]
    }

    fn bytes_append(&mut self, bytes: &[u8]) -> bool {
        self.bytes.append(bytes)
    }

    fn val_types(&self) -> &[ValType] {
        &self.val_types.items[..self.val_types.len]
    }

    fn val_types_append(&mut self, items: &[ValType]) -> bool {
        self.val_types.append(items)
    }

    fn instrs(&self) -> &[Instr] {
        &self.instrs.items[..self.instrs.len]
    }

    fn instrs_append(&mut self, items: &[Instr]) -> bool {
        self.instrs.append(items)
    }

    fn idxs(&self) -> &[u32] {
        &self.idxs.items[..self.idxs.len]
    }

    fn idxs_append<T: Copy + Into<u32>>(&mut self, idxs: &[T]) -> bool {
        self.idxs.append_iter(idxs.iter().map(|&idx| idx.into()))
    }

    fn locals(&self) -> &[Locals] {
        &self.locals.items[..self.locals.len]
    }

    fn locals_append(&mut self, items: &[Locals]) -> bool {
        self.locals.append(items)
    }

    fn func_types(&self) -> &[FuncType] {
        &self.func_types.items[..self.func_types.len]
    }

    fn func_types_append(&mut self, items: &[FuncType]) -> bool {
        self.func_types.append(items)
    }

    fn imports(&self) -> &[Import] {
        &self.imports.items[..self.imports.len]
    }

    fn imports_append(&mut self, items: &[Import]) -> bool {
        self.imports.append(items)
    }

    fn table_types(&self) -> &[TableType] {
        &self.table_types.items[..self.table_types.len]
    }

    fn table_types_append(&mut self, items: &[TableType]) -> bool {
        self.table_types.append(items)
    }

    fn globals(&self) -> &[Global] {
        &self.globals.items[..self.globals.len]
    }

    fn globals_append(&mut self, items: &[Global]) -> bool {
        self.globals.append(items)
    }

    fn exports(&self) -> &[Export] {
        &self.exports.items[..self.exports.len]
    }

    fn exports_append(&mut self, items: &[Export]) -> bool {
        self.exports.append(items)
    }

    fn elems(&self) -> &[Elem] {
        &self.elems.items[..self.elems.len]
    }

    fn elems_append(&mut self, items: &[Elem]) -> bool {
        self.elems.append(items)
    }

    fn codes(&self) -> &[Code] {
        &self.codes.items[..self.codes.len]
    }

    fn codes_append(&mut self, items: &[Code]) -> bool {
        self.codes.append(items)
    }

    fn datas(&self) -> &[Data] {
        &self.datas.items[..self.datas.len]
    }

    fn datas_append(&mut self, items: &[Data]) -> bool {
        self.datas.append(items)
    }
}

#[derive(Debug, Default, Clone)]
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

    fn idxs_append<T: Copy + Into<u32>>(&mut self, idxs: &[T]) -> bool {
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
