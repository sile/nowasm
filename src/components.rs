use crate::decode::Decode;
use crate::execution::{ExecutionError, Value};
use crate::instructions::Instr;
use crate::reader::Reader;
use crate::vector::Vector;
use crate::{DecodeError, Module, VectorFactory};
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

pub struct Name<V: VectorFactory>(V::Vector<u8>);

impl<V: VectorFactory> Name<V> {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let bytes = u8::decode_vector::<V>(reader)?;
        let _ = core::str::from_utf8(&bytes).map_err(DecodeError::InvalidUtf8)?;
        Ok(Self(bytes))
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).expect("unreachable")
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<V: VectorFactory> Debug for Name<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Name").field(&self.as_str()).finish()
    }
}

impl<V: VectorFactory> Clone for Name<V> {
    fn clone(&self) -> Self {
        Self(V::clone_vector(&self.0))
    }
}

pub struct Import<V: VectorFactory> {
    pub module: Name<V>,
    pub name: Name<V>,
    pub desc: ImportDesc,
}

impl<V: VectorFactory> Decode for Import<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let module = Name::decode(reader)?;
        let name = Name::decode(reader)?;
        let desc = ImportDesc::decode(reader)?;
        Ok(Self { module, name, desc })
    }
}

impl<V: VectorFactory> Debug for Import<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Import")
            .field("module", &self.module)
            .field("name", &self.name)
            .field("desc", &self.desc)
            .finish()
    }
}

impl<V: VectorFactory> Clone for Import<V> {
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
    Func(Typeidx),
    Table(TableType),
    Mem(Memtype),
    Global(GlobalType),
}

impl ImportDesc {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x00 => Ok(Self::Func(Typeidx::decode(reader)?)),
            0x01 => Ok(Self::Table(TableType::decode(reader)?)),
            0x02 => Ok(Self::Mem(Memtype::decode(reader)?)),
            0x03 => Ok(Self::Global(GlobalType::decode(reader)?)),
            value => Err(DecodeError::InvalidImportDescTag { value }),
        }
    }
}

pub struct Export<V: VectorFactory> {
    pub name: Name<V>,
    pub desc: ExportDesc,
}

impl<V: VectorFactory> Decode for Export<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let name = Name::decode(reader)?;
        let desc = ExportDesc::decode(reader)?;
        Ok(Self { name, desc })
    }
}

impl<V: VectorFactory> Debug for Export<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Export")
            .field("name", &self.name)
            .field("desc", &self.desc)
            .finish()
    }
}

impl<V: VectorFactory> Clone for Export<V> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            desc: self.desc.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportDesc {
    Func(Funcidx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}

impl ExportDesc {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x00 => Ok(Self::Func(Funcidx::decode(reader)?)),
            0x01 => Ok(Self::Table(TableIdx::decode(reader)?)),
            0x02 => Ok(Self::Mem(MemIdx::decode(reader)?)),
            0x03 => Ok(Self::Global(GlobalIdx::decode(reader)?)),
            value => Err(DecodeError::InvalidExportDescTag { value }),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Typeidx(u32);

impl Typeidx {
    pub const fn get(self) -> usize {
        self.0 as usize
    }
}

// TODO: delete
impl From<Typeidx> for u32 {
    fn from(idx: Typeidx) -> u32 {
        idx.0
    }
}

impl Decode for Typeidx {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Funcidx(u32);

impl Funcidx {
    pub const fn get(self) -> usize {
        self.0 as usize
    }
}

impl Decode for Funcidx {
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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
pub struct Memtype {
    pub limits: Limits,
}

impl Memtype {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        Ok(Self {
            limits: Limits::decode(reader)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GlobalType {
    Const(Valtype),
    Var(Valtype),
}

impl GlobalType {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let t = Valtype::decode(reader)?;
        match reader.read_u8()? {
            0x00 => Ok(Self::Const(t)),
            0x01 => Ok(Self::Var(t)),
            value => Err(DecodeError::InvalidMutabilityFlag { value }),
        }
    }

    pub fn val_type(self) -> Valtype {
        match self {
            Self::Const(t) => t,
            Self::Var(t) => t,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Valtype {
    I32,
    I64,
    F32,
    F64,
}

impl Decode for Valtype {
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

pub struct Func<V: VectorFactory> {
    pub ty: Typeidx,
    pub locals: V::Vector<Valtype>,
    pub body: Expr<V>,
}

impl<V: VectorFactory> Debug for Func<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Func")
            .field("ty", &self.ty)
            .field("locals", &self.locals.as_ref())
            .field("body", &self.body)
            .finish()
    }
}

impl<V: VectorFactory> Clone for Func<V> {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty,
            locals: V::clone_vector(&self.locals),
            body: self.body.clone(),
        }
    }
}

pub struct Functype<V: VectorFactory> {
    pub rt1: ResultType<V>,
    pub rt2: ResultType<V>,
}

impl<V: VectorFactory> Functype<V> {
    pub fn validate_args(
        &self,
        args: &[Value],
        _module: &Module<impl VectorFactory>,
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

impl<V: VectorFactory> Decode for Functype<V> {
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

impl<V: VectorFactory> Debug for Functype<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FuncType")
            .field("rt1", &self.rt1)
            .field("rt2", &self.rt2)
            .finish()
    }
}

impl<V: VectorFactory> Clone for Functype<V> {
    fn clone(&self) -> Self {
        Self {
            rt1: self.rt1.clone(),
            rt2: self.rt2.clone(),
        }
    }
}

pub struct ResultType<V: VectorFactory> {
    pub types: V::Vector<Valtype>,
}

impl<V: VectorFactory> ResultType<V> {
    fn decode_item(reader: &mut Reader) -> Result<Self, DecodeError> {
        let types = Decode::decode_vector::<V>(reader)?;
        Ok(Self { types })
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = Valtype> {
        self.types.iter().copied()
    }
}

impl<V: VectorFactory> Debug for ResultType<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ResultType")
            .field("types", &self.types.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for ResultType<V> {
    fn clone(&self) -> Self {
        Self {
            types: V::clone_vector(&self.types),
        }
    }
}

pub struct Global<V: VectorFactory> {
    pub ty: GlobalType, // TODO: global_type
    pub init: Expr<V>,
}

impl<V: VectorFactory> Global<V> {
    pub fn init(&self) -> Result<Value, ExecutionError> {
        if self.init.len() != 1 {
            return Err(ExecutionError::InvalidGlobalInitializer);
        }
        let Some(instr) = self.init.iter().next() else {
            return Err(ExecutionError::InvalidGlobalInitializer);
        };
        match (self.ty.val_type(), instr) {
            (Valtype::I32, Instr::I32Const(x)) => Ok(Value::I32(*x)),
            (Valtype::I64, Instr::I64Const(x)) => Ok(Value::I64(*x)),
            (Valtype::F32, Instr::F32Const(x)) => Ok(Value::F32(*x)),
            (Valtype::F64, Instr::F64Const(x)) => Ok(Value::F64(*x)),
            _ => Err(ExecutionError::InvalidGlobalInitializer),
        }
    }
}

impl<V: VectorFactory> Decode for Global<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let ty = GlobalType::decode(reader)?;
        let init = Expr::decode(reader)?;
        Ok(Self { ty, init })
    }
}

impl<V: VectorFactory> Debug for Global<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Global")
            .field("ty", &self.ty)
            .field("init", &self.init)
            .finish()
    }
}

impl<V: VectorFactory> Clone for Global<V> {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            init: self.init.clone(),
        }
    }
}

pub struct Expr<V: VectorFactory> {
    instrs: V::Vector<Instr<V>>,
}

impl<V: VectorFactory> Expr<V> {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let mut instrs = V::create_vector(None);
        while reader.peek_u8()? != 0x0b {
            instrs.push(Instr::decode(reader)?);
        }
        reader.read_u8()?;
        Ok(Self { instrs })
    }

    pub fn len(&self) -> usize {
        self.instrs.len()
    }

    pub fn instrs(&self) -> &[Instr<V>] {
        &self.instrs
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = &Instr<V>> {
        self.instrs.iter()
    }
}

impl<V: VectorFactory> Debug for Expr<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Expr")
            .field("instrs", &self.instrs.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for Expr<V> {
    fn clone(&self) -> Self {
        Self {
            instrs: V::clone_vector(&self.instrs),
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

pub struct Elem<V: VectorFactory> {
    pub table: TableIdx,
    pub offset: Expr<V>,
    pub init: V::Vector<Funcidx>,
}

impl<V: VectorFactory> Decode for Elem<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let table = TableIdx::decode(reader)?;
        let offset = Expr::decode(reader)?;
        let init = Funcidx::decode_vector::<V>(reader)?;
        Ok(Self {
            table,
            offset,
            init,
        })
    }
}

impl<V: VectorFactory> Debug for Elem<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Elem")
            .field("table", &self.table)
            .field("offset", &self.offset)
            .field("init", &self.init.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for Elem<V> {
    fn clone(&self) -> Self {
        Self {
            table: self.table,
            offset: self.offset.clone(),
            init: V::clone_vector(&self.init),
        }
    }
}

// TODO: priv
pub struct Code<V: VectorFactory> {
    pub locals: V::Vector<Valtype>,
    pub body: Expr<V>,
}

impl<V: VectorFactory> Code<V> {
    pub fn locals(&self) -> impl '_ + Iterator<Item = Valtype> {
        self.locals.iter().copied()
    }

    pub fn body_iter(&self) -> impl '_ + Iterator<Item = &Instr<V>> {
        self.body.iter()
    }

    pub fn instrs(&self) -> &[Instr<V>] {
        &self.body.instrs
    }
}

impl<V: VectorFactory> Decode for Code<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let code_size = reader.read_usize()?;
        let mut reader = Reader::new(reader.read(code_size)?);
        let mut locals = V::create_vector(None);
        let locals_len = reader.read_usize()?;
        for _ in 0..locals_len {
            let val_types_len = reader.read_usize()?;
            let val_type = Valtype::decode(&mut reader)?;
            for _ in 0..val_types_len {
                locals.push(val_type);
            }
        }
        let body = Expr::decode(&mut reader)?;
        Ok(Self { locals, body })
    }
}

impl<V: VectorFactory> Debug for Code<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Code")
            .field("locals", &self.locals.as_ref())
            .field("body", &self.body)
            .finish()
    }
}

impl<V: VectorFactory> Clone for Code<V> {
    fn clone(&self) -> Self {
        Self {
            locals: V::clone_vector(&self.locals),
            body: self.body.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Empty,
    Val(Valtype),
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
        if let Ok(t) = Valtype::decode(reader) {
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
        // TODO: check sign
        reader.read_integer_s(33).map(Self)
    }
}

pub struct Data<V: VectorFactory> {
    pub data: MemIdx,
    pub offset: Expr<V>,
    pub init: V::Vector<u8>,
}

impl<V: VectorFactory> Decode for Data<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let data = MemIdx::decode(reader)?;
        let offset = Expr::decode(reader)?;
        let init = u8::decode_vector::<V>(reader)?;
        Ok(Self { data, offset, init })
    }
}

impl<V: VectorFactory> Debug for Data<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Data")
            .field("data", &self.data)
            .field("offset", &self.offset)
            .field("init", &self.init.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for Data<V> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            offset: self.offset.clone(),
            init: V::clone_vector(&self.init),
        }
    }
}
