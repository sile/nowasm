use crate::decode::Decode;
use crate::execution::{ExecutionError, Value};
use crate::instructions::Instr;
use crate::reader::Reader;
use crate::vectors::Vector;
use crate::{Allocator, DecodeError, Module};
use core::fmt::{Debug, Formatter};

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

pub use crate::sections::SectionId; // TODO

pub struct Name<A: Allocator>(A::Vector<u8>);

impl<A: Allocator> Name<A> {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let bytes = u8::decode_vector::<A>(reader)?;
        let _ = core::str::from_utf8(bytes.as_ref()).map_err(DecodeError::InvalidUtf8)?;
        Ok(Self(bytes))
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.0.as_ref()).expect("unreachable")
    }

    pub fn len(&self) -> usize {
        self.0.as_ref().len()
    }
}

impl<A: Allocator> Debug for Name<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Name").field(&self.as_str()).finish()
    }
}

impl<A: Allocator> Clone for Name<A> {
    fn clone(&self) -> Self {
        Self(A::clone_vector(&self.0))
    }
}

pub struct Import<A: Allocator> {
    pub module: Name<A>,
    pub name: Name<A>,
    pub desc: ImportDesc,
}

impl<A: Allocator> Decode for Import<A> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let module = Name::decode(reader)?;
        let name = Name::decode(reader)?;
        let desc = ImportDesc::decode(reader)?;
        Ok(Self { module, name, desc })
    }
}

impl<A: Allocator> Debug for Import<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Import")
            .field("module", &self.module)
            .field("name", &self.name)
            .field("desc", &self.desc)
            .finish()
    }
}

impl<A: Allocator> Clone for Import<A> {
    fn clone(&self) -> Self {
        Self {
            module: self.module.clone(),
            name: self.name.clone(),
            desc: self.desc.clone(),
        }
    }
}

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

pub struct Export<A: Allocator> {
    pub name: Name<A>,
    pub desc: ExportDesc,
}

impl<A: Allocator> Decode for Export<A> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let name = Name::decode(reader)?;
        let desc = ExportDesc::decode(reader)?;
        Ok(Self { name, desc })
    }
}

impl<A: Allocator> Debug for Export<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Export")
            .field("name", &self.name)
            .field("desc", &self.desc)
            .finish()
    }
}

impl<A: Allocator> Clone for Export<A> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            desc: self.desc.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}

impl ExportDesc {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x00 => Ok(Self::Func(FuncIdx::decode(reader)?)),
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

impl From<TypeIdx> for u32 {
    fn from(idx: TypeIdx) -> u32 {
        idx.0
    }
}

impl Decode for TypeIdx {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FuncIdx(u32);

impl FuncIdx {
    pub fn get(self) -> u32 {
        self.0
    }
}

impl FuncIdx {
    pub fn get_type<A: Allocator>(self, module: &Module<A>) -> Option<&FuncType<A>> {
        let type_idx = module
            .function_section()
            .idxs
            .as_ref()
            .get(self.0 as usize)?;
        module
            .type_section()
            .types
            .as_ref()
            .get(type_idx.0 as usize)
    }

    pub fn get_code<A: Allocator>(self, module: &Module<A>) -> Option<&Code<A>> {
        module.code_section().codes.as_ref().get(self.0 as usize)
    }
}

impl Decode for FuncIdx {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
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

    pub fn as_usize(self) -> usize {
        self.0 as usize
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LabelIdx(u32);

impl LabelIdx {
    pub fn new(v: u32) -> Self {
        Self(v)
    }

    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TableType {
    pub limits: Limits,
}

impl Decode for TableType {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let elem_type = reader.read_u8()?;
        if elem_type != 0x70 {
            return Err(DecodeError::InvalidElemType { value: elem_type });
        }
        let limits = Limits::decode(reader)?;
        Ok(Self { limits })
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

impl Decode for ValType {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x7f => Ok(Self::I32),
            0x7e => Ok(Self::I64),
            0x7d => Ok(Self::F32),
            0x7c => Ok(Self::F64),
            value => Err(DecodeError::InvalidValType { value }),
        }
    }
}

pub struct FuncType<A: Allocator> {
    pub rt1: ResultType<A>,
    pub rt2: ResultType<A>,
}

impl<A: Allocator> FuncType<A> {
    pub fn validate_args(
        &self,
        args: &[Value],
        _module: &Module<impl Allocator>,
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

    pub fn args_len(&self) -> usize {
        self.rt1.len()
    }

    pub fn return_arity(&self) -> usize {
        self.rt2.len()
    }
}

impl<A: Allocator> Decode for FuncType<A> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        if tag != 0x60 {
            return Err(DecodeError::InvalidFuncTypeTag { value: tag });
        }
        let rt1 = ResultType::decode_item(reader)?;
        let rt2 = ResultType::decode_item(reader)?;
        Ok(Self { rt1, rt2 })
    }
}

impl<A: Allocator> Debug for FuncType<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FuncType")
            .field("rt1", &self.rt1)
            .field("rt2", &self.rt2)
            .finish()
    }
}

impl<A: Allocator> Clone for FuncType<A> {
    fn clone(&self) -> Self {
        Self {
            rt1: self.rt1.clone(),
            rt2: self.rt2.clone(),
        }
    }
}

pub struct ResultType<A: Allocator> {
    pub types: A::Vector<ValType>,
}

impl<A: Allocator> ResultType<A> {
    fn decode_item(reader: &mut Reader) -> Result<Self, DecodeError> {
        let types = Decode::decode_vector::<A>(reader)?;
        Ok(Self { types })
    }

    pub fn len(&self) -> usize {
        self.types.as_ref().len()
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = ValType> {
        self.types.as_ref().iter().copied()
    }
}

impl<A: Allocator> Debug for ResultType<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ResultType")
            .field("types", &self.types.as_ref())
            .finish()
    }
}

impl<A: Allocator> Clone for ResultType<A> {
    fn clone(&self) -> Self {
        Self {
            types: A::clone_vector(&self.types),
        }
    }
}

pub struct Global<A: Allocator> {
    pub ty: GlobalType, // TODO: global_type
    pub init: Expr<A>,
}

impl<A: Allocator> Global<A> {
    pub fn init(&self) -> Result<Value, ExecutionError> {
        if self.init.len() != 1 {
            return Err(ExecutionError::InvalidGlobalInitializer);
        }
        let Some(instr) = self.init.iter().next() else {
            return Err(ExecutionError::InvalidGlobalInitializer);
        };
        match (self.ty.val_type(), instr) {
            (ValType::I32, Instr::I32Const(x)) => Ok(Value::I32(*x)),
            (ValType::I64, Instr::I64Const(x)) => Ok(Value::I64(*x)),
            (ValType::F32, Instr::F32Const(x)) => Ok(Value::F32(*x)),
            (ValType::F64, Instr::F64Const(x)) => Ok(Value::F64(*x)),
            _ => Err(ExecutionError::InvalidGlobalInitializer),
        }
    }
}

impl<A: Allocator> Decode for Global<A> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let ty = GlobalType::decode(reader)?;
        let init = Expr::decode(reader)?;
        Ok(Self { ty, init })
    }
}

impl<A: Allocator> Debug for Global<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Global")
            .field("ty", &self.ty)
            .field("init", &self.init)
            .finish()
    }
}

impl<A: Allocator> Clone for Global<A> {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            init: self.init.clone(),
        }
    }
}

pub struct Expr<A: Allocator> {
    instrs: A::Vector<Instr<A>>,
}

impl<A: Allocator> Expr<A> {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let mut instrs = A::allocate_vector();
        while reader.peek_u8()? != 0x0b {
            instrs.push(Instr::decode(reader)?);
        }
        reader.read_u8()?;
        Ok(Self { instrs })
    }

    pub fn len(&self) -> usize {
        self.instrs.as_ref().len()
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = &Instr<A>> {
        self.instrs.as_ref().iter()
    }
}

impl<A: Allocator> Debug for Expr<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Expr")
            .field("instrs", &self.instrs.as_ref())
            .finish()
    }
}

impl<A: Allocator> Clone for Expr<A> {
    fn clone(&self) -> Self {
        Self {
            instrs: A::clone_vector(&self.instrs),
        }
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

pub struct Elem<A: Allocator> {
    pub table: TableIdx,
    pub offset: Expr<A>,
    pub init: A::Vector<FuncIdx>,
}

impl<A: Allocator> Decode for Elem<A> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let table = TableIdx::decode(reader)?;
        let offset = Expr::decode(reader)?;
        let init = FuncIdx::decode_vector::<A>(reader)?;
        Ok(Self {
            table,
            offset,
            init,
        })
    }
}

impl<A: Allocator> Debug for Elem<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Elem")
            .field("table", &self.table)
            .field("offset", &self.offset)
            .field("init", &self.init.as_ref())
            .finish()
    }
}

impl<A: Allocator> Clone for Elem<A> {
    fn clone(&self) -> Self {
        Self {
            table: self.table,
            offset: self.offset.clone(),
            init: A::clone_vector(&self.init),
        }
    }
}

pub struct Code<A: Allocator> {
    pub locals: A::Vector<ValType>,
    pub body: Expr<A>,
}

impl<A: Allocator> Code<A> {
    pub fn locals(&self) -> impl '_ + Iterator<Item = ValType> {
        self.locals.as_ref().iter().copied()
    }

    pub fn body_iter(&self) -> impl '_ + Iterator<Item = &Instr<A>> {
        self.body.iter()
    }

    pub fn instrs(&self) -> &[Instr<A>] {
        self.body.instrs.as_ref()
    }
}

impl<A: Allocator> Decode for Code<A> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let code_size = reader.read_usize()?;
        let mut reader = Reader::new(reader.read(code_size)?);
        let mut locals = A::allocate_vector();
        let locals_len = reader.read_usize()?;
        for _ in 0..locals_len {
            let val_types_len = reader.read_usize()?;
            let val_type = ValType::decode(&mut reader)?;
            for _ in 0..val_types_len {
                locals.push(val_type);
            }
        }
        let body = Expr::decode(&mut reader)?;
        Ok(Self { locals, body })
    }
}

impl<A: Allocator> Debug for Code<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Code")
            .field("locals", &self.locals.as_ref())
            .field("body", &self.body)
            .finish()
    }
}

impl<A: Allocator> Clone for Code<A> {
    fn clone(&self) -> Self {
        Self {
            locals: A::clone_vector(&self.locals),
            body: self.body.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Empty,
    Val(ValType),
    TypeIndex(S33),
}

impl BlockType {
    pub fn arity(self) -> usize {
        match self {
            BlockType::Empty => 0,
            BlockType::Val(_) => 1,
            BlockType::TypeIndex(_) => todo!(),
        }
    }

    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        if reader.read_u8()? == 0x40 {
            return Ok(Self::Empty);
        }

        reader.unread_u8();
        if let Ok(t) = ValType::decode(reader) {
            return Ok(Self::Val(t));
        }

        reader.unread_u8();
        Ok(Self::TypeIndex(S33::decode(reader)?)) // TODO: n>0 check
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S33(i64);

impl S33 {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_integer_s(33).map(Self)
    }
}

#[derive(Debug, Clone)]
pub struct Data<A: Allocator> {
    pub data: MemIdx,
    pub offset: Expr<A>,
    pub init: A::Vector<u8>,
}

impl<A: Allocator> Decode for Data<A> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let data = MemIdx::decode(reader)?;
        let offset = Expr::decode(reader)?;
        let init = u8::decode_vector::<A>(reader)?;
        Ok(Self { data, offset, init })
    }
}
