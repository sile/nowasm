use crate::execution::{ExecutionError, Value};
use crate::instructions::Instr;
use crate::reader::Reader;
use crate::vectors::{VectorItem, VectorKind, Vectors};
use crate::{Allocator, DecodeError, DecodeVector, Module};

#[derive(Debug)]
pub struct Magic;

impl Magic {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let mut value = [0; 4];
        reader.read_exact(&mut value)?;
        if value != *b"\0asm" {
            return Err(DecodeError::InvalidMagic { value });
        }
        Ok(Self)
    }
}

#[derive(Debug)]
pub struct Version;

impl Version {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let mut value = [0; 4];
        reader.read_exact(&mut value)?;
        if value != [1, 0, 0, 0] {
            return Err(DecodeError::InvalidVersion { value });
        }
        Ok(Self)
    }
}

pub use crate::sections::SectionId;

#[derive(Debug, Clone)]
pub struct Name<A: Allocator> {
    bytes: A::Vector<u8>, // TODO: A::String
}

impl<A: Allocator> Name<A> {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let start = vectors.bytes().len();
        let len = reader.read_usize()?;
        let name = reader.read(len)?;
        let _ = core::str::from_utf8(name).map_err(DecodeError::InvalidUtf8)?;
        if !vectors.bytes_append(name) {
            return Err(DecodeError::FullVector {
                kind: VectorKind::Bytes,
            });
        }
        Ok(Self { start, len })
    }

    pub fn as_str(self, vectors: &impl Vectors) -> Option<&str> {
        let bytes = vectors.bytes();
        let start = self.start;
        let end = start + self.len;
        if bytes.len() < end {
            return None;
        }
        core::str::from_utf8(&bytes[start..end]).ok()
    }

    pub fn len(self) -> usize {
        self.len
    }
}

#[derive(Debug, Clone)]
pub struct Import<A: Allocator> {
    pub module: Name<A>,
    pub name: Name<A>,
    pub desc: ImportDesc,
}

impl<A: Allocator> Import<A> {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let module = Name::decode(reader, vectors)?;
        let name = Name::decode(reader, vectors)?;
        let desc = ImportDesc::decode(reader)?;
        Ok(Self { module, name, desc })
    }
}

impl<A: Allocator> VectorItem for Import<A> {}

#[derive(Debug, Clone, Copy)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

impl ImportDesc {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x00 => Ok(Self::Func(TypeIdx::decode(reader)?)),
            0x01 => Ok(Self::Table(TableType::decode(reader)?)),
            0x02 => Ok(Self::Mem(MemType::decode(reader)?)),
            0x03 => Ok(Self::Global(GlobalType::decode(reader)?)),
            value => Err(DecodeError::InvalidImportDescTag { value }),
        }
    }
}

impl Default for ImportDesc {
    fn default() -> Self {
        Self::Func(Default::default())
    }
}

#[derive(Debug, Clone)]
pub struct Export<A: Allocator> {
    pub name: Name<A>,
    pub desc: ExportDesc,
}

impl<A: Allocator> Export<A> {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let name = Name::decode(reader, vectors)?;
        let desc = ExportDesc::decode(reader)?;
        Ok(Self { name, desc })
    }
}

// TODO
// impl VectorItem for Export {
//     fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError> {
//         Self::decode(reader, vectors)
//     }

//     fn append<V: Vectors>(items: &[Self], vectors: &mut V) -> Result<usize, DecodeError> {
//         if !vectors.exports_append(items) {
//             return Err(DecodeError::FullVector {
//                 kind: VectorKind::Exports,
//             });
//         }
//         Ok(vectors.exports().len())
//     }

//     fn get(index: usize, vectors: &impl Vectors) -> Option<Self> {
//         vectors.exports().get(index).copied()
//     }
// }

#[derive(Debug, Clone, Copy)]
pub enum ExportDesc {
    Func(TypeIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}

impl ExportDesc {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x00 => Ok(Self::Func(TypeIdx::decode(reader)?)),
            0x01 => Ok(Self::Table(TableIdx::decode(reader)?)),
            0x02 => Ok(Self::Mem(MemIdx::decode(reader)?)),
            0x03 => Ok(Self::Global(GlobalIdx::decode(reader)?)),
            value => Err(DecodeError::InvalidExportDescTag { value }),
        }
    }
}

impl Default for ExportDesc {
    fn default() -> Self {
        Self::Func(Default::default())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TypeIdx(u32);

impl TypeIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }

    pub fn get_type<A: Allocator>(self, module: &Module<impl Vectors, A>) -> Option<&FuncType<A>> {
        let type_idx = module
            .function_section()
            .idxs
            .get(self.0 as usize, module.vectors())?;
        module
            .type_section()
            .types
            .as_ref()
            .get(type_idx.0 as usize)
    }

    pub fn get_code(self, module: &Module<impl Vectors, impl Allocator>) -> Option<Code> {
        let code = module
            .code_section()
            .codes
            .get(self.0 as usize, module.vectors())?;
        Some(code)
    }
}

impl From<TypeIdx> for u32 {
    fn from(idx: TypeIdx) -> u32 {
        idx.0
    }
}

impl VectorItem for TypeIdx {
    fn decode<V: Vectors>(reader: &mut Reader, _vectors: &mut V) -> Result<Self, DecodeError> {
        Self::decode(reader)
    }

    fn append<V: Vectors>(items: &[Self], vectors: &mut V) -> Result<usize, DecodeError> {
        if !vectors.idxs_append(items) {
            return Err(DecodeError::FullVector {
                kind: VectorKind::Idxs,
            });
        }
        Ok(vectors.idxs().len())
    }

    fn get(index: usize, vectors: &impl Vectors) -> Option<Self> {
        vectors.idxs().get(index).copied().map(Self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FuncIdx(u32);

impl FuncIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TableIdx;

impl TableIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let i = reader.read_u32()?;
        if i != 0 {
            return Err(DecodeError::InvalidTableIdx { value: i });
        }
        Ok(Self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MemIdx;

impl MemIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let i = reader.read_u32()?;
        if i != 0 {
            return Err(DecodeError::InvalidMemIdx { value: i });
        }
        Ok(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalIdx(u32);

impl GlobalIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocalIdx(u32);

impl LocalIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LabelIdx(u32);

impl LabelIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TableType {
    pub limits: Limits,
}

impl TableType {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let elem_type = reader.read_u8()?;
        if elem_type != 0x70 {
            return Err(DecodeError::InvalidElemType { value: elem_type });
        }
        let limits = Limits::decode(reader)?;
        Ok(Self { limits })
    }
}

impl VectorItem for TableType {
    fn decode<V: Vectors>(reader: &mut Reader, _vectors: &mut V) -> Result<Self, DecodeError> {
        Self::decode(reader)
    }

    fn append<V: Vectors>(items: &[Self], vectors: &mut V) -> Result<usize, DecodeError> {
        if !vectors.table_types_append(items) {
            return Err(DecodeError::FullVector {
                kind: VectorKind::TableTypes,
            });
        }
        Ok(vectors.table_types().len())
    }

    fn get(index: usize, vectors: &impl Vectors) -> Option<Self> {
        vectors.table_types().get(index).copied()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl Limits {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x00 => {
                let min = reader.read_u32()?;
                Ok(Self { min, max: None })
            }
            0x01 => {
                let min = reader.read_u32()?;
                let max = Some(reader.read_u32()?);
                Ok(Self { min, max })
            }
            value => Err(DecodeError::InvalidLimitsFlag { value }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemType {
    pub limits: Limits,
}

impl MemType {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        Ok(Self {
            limits: Limits::decode(reader)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GlobalType {
    Const(ValType),
    Var(ValType),
}

impl GlobalType {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let t = ValType::decode(reader)?;
        match reader.read_u8()? {
            0x00 => Ok(Self::Const(t)),
            0x01 => Ok(Self::Var(t)),
            value => Err(DecodeError::InvalidMutabilityFlag { value }),
        }
    }

    pub fn val_type(self) -> ValType {
        match self {
            Self::Const(t) => t,
            Self::Var(t) => t,
        }
    }
}

impl Default for GlobalType {
    fn default() -> Self {
        Self::Const(ValType::I32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
}

impl ValType {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x7f => Ok(Self::I32),
            0x7e => Ok(Self::I64),
            0x7d => Ok(Self::F32),
            0x7c => Ok(Self::F64),
            value => Err(DecodeError::InvalidValType { value }),
        }
    }
}

impl<A: Allocator> DecodeVector<A> for ValType {
    fn decode_item(reader: &mut Reader) -> Result<Self, DecodeError> {
        Self::decode(reader)
    }
}

#[derive(Debug, Clone)]
pub struct FuncType<A: Allocator> {
    pub rt1: ResultType<A>,
    pub rt2: ResultType<A>,
}

impl<A: Allocator> FuncType<A> {
    pub fn validate_args(
        &self,
        args: &[Value],
        _module: &Module<impl Vectors, impl Allocator>,
    ) -> Result<(), ExecutionError> {
        if args.len() != self.rt1.len() {
            return Err(ExecutionError::InvalidFuncArgs);
        }

        for (ty, val) in self.rt1.iter().zip(args.iter()) {
            if ty != val.ty() {
                return Err(ExecutionError::InvalidFuncArgs);
            }
        }

        Ok(())
    }
}

impl<A: Allocator> DecodeVector<A> for FuncType<A> {
    fn decode_item(reader: &mut Reader) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        if tag != 0x60 {
            return Err(DecodeError::InvalidFuncTypeTag { value: tag });
        }
        let rt1 = ResultType::decode_item(reader)?;
        let rt2 = ResultType::decode_item(reader)?;
        Ok(Self { rt1, rt2 })
    }
}

impl<A: Allocator> VectorItem for FuncType<A> {
    fn decode<V: Vectors>(_reader: &mut Reader, _vectors: &mut V) -> Result<Self, DecodeError> {
        todo!()
    }

    fn append<V: Vectors>(_items: &[Self], _vectors: &mut V) -> Result<usize, DecodeError> {
        todo!()
    }

    fn get(_index: usize, _vectors: &impl Vectors) -> Option<Self> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct ResultType<A: Allocator> {
    pub types: A::Vector<ValType>,
}

impl<A: Allocator> ResultType<A> {
    fn decode_item(reader: &mut Reader) -> Result<Self, DecodeError> {
        let types = DecodeVector::<A>::decode_vector(reader)?;
        Ok(Self { types })
    }

    pub fn len(&self) -> usize {
        self.types.as_ref().len()
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = ValType> {
        self.types.as_ref().iter().copied()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Global {
    pub ty: GlobalType,
    pub init: Expr,
}

impl Global {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let ty = GlobalType::decode(reader)?;
        let init = Expr::decode(reader, vectors)?;
        Ok(Self { ty, init })
    }

    pub fn init(
        &self,
        module: &Module<impl Vectors, impl Allocator>,
    ) -> Result<Value, ExecutionError> {
        if self.init.len != 1 {
            return Err(ExecutionError::InvalidGlobalInitializer);
        }
        let Some(instr) = self.init.iter(module).next() else {
            return Err(ExecutionError::InvalidGlobalInitializer);
        };
        match (self.ty.val_type(), instr) {
            (ValType::I32, Instr::I32Const(x)) => Ok(Value::I32(x)),
            (ValType::I64, Instr::I64Const(x)) => Ok(Value::I64(x)),
            (ValType::F32, Instr::F32Const(x)) => Ok(Value::F32(x)),
            (ValType::F64, Instr::F64Const(x)) => Ok(Value::F64(x)),
            _ => Err(ExecutionError::InvalidGlobalInitializer),
        }
    }
}

impl VectorItem for Global {
    fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError> {
        Self::decode(reader, vectors)
    }

    fn append<V: Vectors>(items: &[Self], vectors: &mut V) -> Result<usize, DecodeError> {
        if !vectors.globals_append(items) {
            return Err(DecodeError::FullVector {
                kind: VectorKind::Globals,
            });
        }
        Ok(vectors.globals().len())
    }

    fn get(index: usize, vectors: &impl Vectors) -> Option<Self> {
        vectors.globals().get(index).copied()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Expr {
    pub start: usize, // TODO
    len: usize,
}

impl Expr {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let start = vectors.instrs().len();
        while reader.peek_u8()? != 0x0b {
            let instr = Instr::decode(reader, vectors)?;
            if !vectors.instrs_append(&[instr]) {
                return Err(DecodeError::FullVector {
                    kind: VectorKind::Instrs,
                });
            }
        }
        reader.read_u8()?;
        let end = vectors.instrs().len();
        let len = end - start;
        Ok(Self { start, len })
    }

    pub fn len(self) -> usize {
        self.len
    }

    pub fn iter(
        self,
        module: &Module<impl Vectors, impl Allocator>,
    ) -> impl '_ + Iterator<Item = Instr> {
        module.vectors().instrs()[self.start..self.start + self.len]
            .iter()
            .copied()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

impl MemArg {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let align = reader.read_u32()?;
        let offset = reader.read_u32()?;
        Ok(Self { align, offset })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Elem {
    pub table: TableIdx,
    pub offset: Expr,
    pub init: FuncIdxVec,
}

impl Elem {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let table = TableIdx::decode(reader)?;
        let offset = Expr::decode(reader, vectors)?;
        let init = FuncIdxVec::decode(reader, vectors)?;
        Ok(Self {
            table,
            offset,
            init,
        })
    }
}

impl VectorItem for Elem {
    fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError> {
        Self::decode(reader, vectors)
    }

    fn append<V: Vectors>(items: &[Self], vectors: &mut V) -> Result<usize, DecodeError> {
        if !vectors.elems_append(items) {
            return Err(DecodeError::FullVector {
                kind: VectorKind::Elems,
            });
        }
        Ok(vectors.elems().len())
    }

    fn get(index: usize, vectors: &impl Vectors) -> Option<Self> {
        vectors.elems().get(index).copied()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FuncIdxVec {
    pub start: usize,
    len: usize,
}

impl FuncIdxVec {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let start = vectors.idxs().len();
        let len = reader.read_usize()?;
        for _ in 0..len {
            let idx = FuncIdx::decode(reader)?;
            if !vectors.idxs_append(&[idx.0]) {
                return Err(DecodeError::FullVector {
                    kind: VectorKind::Idxs,
                });
            }
        }
        Ok(Self { start, len })
    }

    pub fn len(self) -> usize {
        self.len
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Code {
    // TODO: func: Func
    pub locals_start: usize, // TODO: use VectorSlice
    pub locals_len: usize,
    pub body: Expr,
}

impl Code {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let code_size = reader.read_usize()?;
        let mut reader = Reader::new(reader.read(code_size)?);

        let locals_start = vectors.locals().len();
        let locals_len = reader.read_usize()?;
        for _ in 0..locals_len {
            let locals = Locals::decode(&mut reader)?;
            if !vectors.locals_append(&[locals]) {
                return Err(DecodeError::FullVector {
                    kind: VectorKind::Locals,
                });
            }
        }
        let body = Expr::decode(&mut reader, vectors)?;
        Ok(Self {
            locals_start,
            locals_len,
            body,
        })
    }

    pub fn locals(
        self,
        module: &Module<impl Vectors, impl Allocator>,
    ) -> impl '_ + Iterator<Item = ValType> {
        module.vectors().locals()[self.locals_start..self.locals_start + self.locals_len]
            .iter()
            .copied()
            .flat_map(|locals| std::iter::repeat(locals.t).take(locals.n as usize))
    }

    pub fn instrs(
        self,
        module: &Module<impl Vectors, impl Allocator>,
    ) -> impl '_ + Iterator<Item = Instr> {
        self.body.iter(module)
    }
}

impl VectorItem for Code {
    fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError> {
        Self::decode(reader, vectors)
    }

    fn append<V: Vectors>(items: &[Self], vectors: &mut V) -> Result<usize, DecodeError> {
        if !vectors.codes_append(items) {
            return Err(DecodeError::FullVector {
                kind: VectorKind::Codes,
            });
        }
        Ok(vectors.codes().len())
    }

    fn get(index: usize, vectors: &impl Vectors) -> Option<Self> {
        vectors.codes().get(index).copied()
    }
}

// TODO: flatten(?)
#[derive(Debug, Clone, Copy)]
pub struct Locals {
    pub n: u32,
    pub t: ValType,
}

impl Locals {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let n = reader.read_u32()?;
        let t = ValType::decode(reader)?;
        Ok(Self { n, t })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Empty,
    Val(ValType),
    TypeIndex(S33),
}

impl BlockType {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        if reader.read_u8()? == 0x40 {
            return Ok(Self::Empty);
        }

        reader.unread_u8();
        if let Ok(t) = ValType::decode(reader) {
            return Ok(Self::Val(t));
        }

        reader.unread_u8();
        Ok(Self::TypeIndex(S33::decode(reader)?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S33(i64);

impl S33 {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_integer_s(33).map(Self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Data {
    pub data: MemIdx,
    pub offset: Expr,
    pub init_start: usize,
    pub init_end: usize,
}

impl Data {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let data = MemIdx::decode(reader)?;
        let offset = Expr::decode(reader, vectors)?;
        let init_start = vectors.bytes().len();
        let init_len = reader.read_usize()?;
        vectors.bytes_append(reader.read(init_len)?);
        let init_end = vectors.bytes().len();
        Ok(Self {
            data,
            offset,
            init_start,
            init_end,
        })
    }
}

impl VectorItem for Data {
    fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError> {
        Self::decode(reader, vectors)
    }

    fn append<V: Vectors>(items: &[Self], vectors: &mut V) -> Result<usize, DecodeError> {
        if !vectors.datas_append(items) {
            return Err(DecodeError::FullVector {
                kind: VectorKind::Datas,
            });
        }
        Ok(vectors.datas().len())
    }

    fn get(index: usize, vectors: &impl Vectors) -> Option<Self> {
        vectors.datas().get(index).copied()
    }
}
