use crate::decode::Decode;
use crate::execute::{ExecuteError, Val};
use crate::instructions::Instr;
use crate::reader::Reader;
use crate::vector::Vector;
use crate::{DecodeError, Module, VectorFactory, PAGE_SIZE};
use core::fmt::{Debug, Formatter};

pub struct Name<V: VectorFactory>(V::Vector<u8>);

impl<V: VectorFactory> Name<V> {
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).expect("unreachable")
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<V: VectorFactory> Decode<V> for Name<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let bytes = Decode::<V>::decode_vector(reader)?;
        let _ = core::str::from_utf8(&bytes).map_err(DecodeError::InvalidUtf8)?;
        Ok(Self(bytes))
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
    pub desc: Importdesc,
}

impl<V: VectorFactory> Decode<V> for Import<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let module = Name::decode(reader)?;
        let name = Name::decode(reader)?;
        let desc = Decode::<V>::decode(reader)?;
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
pub enum Importdesc {
    Func(Typeidx),
    Table(Tabletype),
    Mem(Memtype),
    Global(Globaltype),
}

impl<V: VectorFactory> Decode<V> for Importdesc {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x00 => Ok(Self::Func(Decode::<V>::decode(reader)?)),
            0x01 => Ok(Self::Table(Decode::<V>::decode(reader)?)),
            0x02 => Ok(Self::Mem(Decode::<V>::decode(reader)?)),
            0x03 => Ok(Self::Global(Decode::<V>::decode(reader)?)),
            value => Err(DecodeError::InvalidImportDescTag { value }),
        }
    }
}

pub struct Export<V: VectorFactory> {
    pub name: Name<V>,
    pub desc: Exportdesc,
}

impl<V: VectorFactory> Decode<V> for Export<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let name = Name::decode(reader)?;
        let desc = Decode::<V>::decode(reader)?;
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
pub enum Exportdesc {
    Func(Funcidx),
    Table(Tableidx),
    Mem(Memidx),
    Global(Globalidx),
}

impl<V: VectorFactory> Decode<V> for Exportdesc {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0x00 => Ok(Self::Func(Decode::<V>::decode(reader)?)),
            0x01 => Ok(Self::Table(Decode::<V>::decode(reader)?)),
            0x02 => Ok(Self::Mem(Decode::<V>::decode(reader)?)),
            0x03 => Ok(Self::Global(Decode::<V>::decode(reader)?)),
            value => Err(DecodeError::InvalidExportDescTag { value }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Typeidx(u32);

impl Typeidx {
    pub const fn get(self) -> usize {
        self.0 as usize
    }
}

impl<V: VectorFactory> Decode<V> for Typeidx {
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

impl<V: VectorFactory> Decode<V> for Funcidx {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tableidx;

impl<V: VectorFactory> Decode<V> for Tableidx {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let i = reader.read_u32()?;
        if i != 0 {
            return Err(DecodeError::InvalidTableIdx { value: i });
        }
        Ok(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Memidx;

impl<V: VectorFactory> Decode<V> for Memidx {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let i = reader.read_u32()?;
        if i != 0 {
            return Err(DecodeError::InvalidMemIdx { value: i });
        }
        Ok(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Globalidx(u32);

impl Globalidx {
    pub const fn get(self) -> usize {
        self.0 as usize
    }
}

impl<V: VectorFactory> Decode<V> for Globalidx {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Localidx(u32);

impl Localidx {
    pub const fn get(self) -> usize {
        self.0 as usize
    }
}

impl<V: VectorFactory> Decode<V> for Localidx {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Labelidx(u32);

impl Labelidx {
    pub const fn new(v: u32) -> Self {
        Self(v)
    }

    pub const fn get(self) -> usize {
        self.0 as usize
    }
}

impl<V: VectorFactory> Decode<V> for Labelidx {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tabletype {
    pub elemtype: Elemtype,
    pub limits: Limits,
}

impl<V: VectorFactory> Decode<V> for Tabletype {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let elemtype = Decode::<V>::decode(reader)?;
        let limits = Decode::<V>::decode(reader)?;
        Ok(Self { elemtype, limits })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Elemtype;

impl<V: VectorFactory> Decode<V> for Elemtype {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let elem_type = reader.read_u8()?;
        if elem_type != 0x70 {
            return Err(DecodeError::InvalidElemType { value: elem_type });
        }
        Ok(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl<V: VectorFactory> Decode<V> for Limits {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
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
    pub fn min_bytes(self) -> usize {
        self.limits.min as usize * PAGE_SIZE
    }

    pub fn max_bytes(self) -> Option<usize> {
        self.limits.max.map(|max| max as usize * PAGE_SIZE)
    }

    pub fn contains(self, bytes: usize) -> bool {
        if let Some(max) = self.max_bytes() {
            self.min_bytes() <= bytes && bytes <= max
        } else {
            self.min_bytes() <= bytes
        }
    }
}

impl<V: VectorFactory> Decode<V> for Memtype {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        Ok(Self {
            limits: Decode::<V>::decode(reader)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Globaltype {
    Const(Valtype),
    Var(Valtype),
}

impl Globaltype {
    pub fn val_type(self) -> Valtype {
        match self {
            Self::Const(t) => t,
            Self::Var(t) => t,
        }
    }
}

impl<V: VectorFactory> Decode<V> for Globaltype {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let t = Decode::<V>::decode(reader)?;
        match reader.read_u8()? {
            0x00 => Ok(Self::Const(t)),
            0x01 => Ok(Self::Var(t)),
            value => Err(DecodeError::InvalidMutabilityFlag { value }),
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

impl<V: VectorFactory> Decode<V> for Valtype {
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
    pub params: V::Vector<Valtype>,
    pub result: Resulttype,
}

impl<V: VectorFactory> Functype<V> {
    pub fn validate_args(
        &self,
        args: &[Val],
        _module: &Module<impl VectorFactory>,
    ) -> Result<(), ExecuteError> {
        if args.len() != self.params.len() {
            return Err(ExecuteError::InvalidFuncArgs);
        }

        for (&expected_type, actual_value) in self.params.iter().zip(args.iter()) {
            if expected_type != actual_value.ty() {
                return Err(ExecuteError::InvalidFuncArgs);
            }
        }

        Ok(())
    }
}

impl<V: VectorFactory> Decode<V> for Functype<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        if tag != 0x60 {
            return Err(DecodeError::InvalidFuncTypeTag { value: tag });
        }
        let params = Decode::<V>::decode_vector(reader)?;
        let result = Decode::<V>::decode(reader)?;
        Ok(Self { params, result })
    }
}

impl<V: VectorFactory> Debug for Functype<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Functype")
            .field("params", &self.params.as_ref())
            .field("result", &self.result)
            .finish()
    }
}

impl<V: VectorFactory> Clone for Functype<V> {
    fn clone(&self) -> Self {
        Self {
            params: V::clone_vector(&self.params),
            result: self.result.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Resulttype(Option<Valtype>);

impl Resulttype {
    pub fn len(self) -> usize {
        self.0.is_some() as usize
    }

    pub fn get(self) -> Option<Valtype> {
        self.0
    }
}

impl<V: VectorFactory> Decode<V> for Resulttype {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let size = reader.read_usize()?;
        match size {
            0 => Ok(Self(None)),
            1 => Ok(Self(Some(Decode::<V>::decode(reader)?))),
            _ => Err(DecodeError::InvalidResultArity { value: size }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Global {
    pub ty: Globaltype,
    pub init: ConstantExpr,
}

impl Global {
    pub fn init(&self /* TODO: imported globals */) -> Result<Val, ExecuteError> {
        match (self.ty.val_type(), self.init) {
            (Valtype::I32, ConstantExpr::I32(x)) => Ok(Val::I32(x)),
            (Valtype::I64, ConstantExpr::I64(x)) => Ok(Val::I64(x)),
            (Valtype::F32, ConstantExpr::F32(x)) => Ok(Val::F32(x)),
            (Valtype::F64, ConstantExpr::F64(x)) => Ok(Val::F64(x)),
            (_ty, ConstantExpr::Global(_idx)) => {
                // TODO
                Err(ExecuteError::InvalidGlobalInitializer)
            }
            _ => Err(ExecuteError::InvalidGlobalInitializer),
        }
    }
}

impl<V: VectorFactory> Decode<V> for Global {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let ty = Decode::<V>::decode(reader)?;
        let init = Decode::<V>::decode(reader)?;
        Ok(Self { ty, init })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum I32ConstantExpr {
    I32(i32),
    Global(Globalidx),
}

impl I32ConstantExpr {
    pub fn get<V: VectorFactory>(self, globals: &[Val], module: &Module<V>) -> Option<i32> {
        match self {
            Self::I32(v) => Some(v),
            Self::Global(idx) => {
                let Global {
                    ty: Globaltype::Const(Valtype::I32),
                    ..
                } = module.globals().get(idx.get())?
                else {
                    return None;
                };
                let Val::I32(v) = globals.get(idx.get()).copied()? else {
                    return None;
                };
                Some(v)
            }
        }
    }
}

impl<V: VectorFactory> Decode<V> for I32ConstantExpr {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let expr = Expr::<V>::decode(reader)?;
        if expr.instrs().len() != 1 {
            return Err(DecodeError::UnexpectedExpr);
        }
        match &expr.instrs()[0] {
            Instr::I32Const(x) => Ok(Self::I32(*x)),
            Instr::GlobalGet(x) => Ok(Self::Global(*x)),
            _ => Err(DecodeError::UnexpectedExpr),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConstantExpr {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Global(Globalidx),
}

impl<V: VectorFactory> Decode<V> for ConstantExpr {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let expr = Expr::<V>::decode(reader)?;
        if expr.instrs().len() != 1 {
            return Err(DecodeError::UnexpectedExpr);
        }
        match &expr.instrs()[0] {
            Instr::I32Const(x) => Ok(Self::I32(*x)),
            Instr::I64Const(x) => Ok(Self::I64(*x)),
            Instr::F32Const(x) => Ok(Self::F32(*x)),
            Instr::F64Const(x) => Ok(Self::F64(*x)),
            Instr::GlobalGet(x) => Ok(Self::Global(*x)),
            _ => Err(DecodeError::UnexpectedExpr),
        }
    }
}

pub struct Expr<V: VectorFactory> {
    instrs: V::Vector<Instr<V>>,
}

impl<V: VectorFactory> Expr<V> {
    pub fn instrs(&self) -> &[Instr<V>] {
        &self.instrs
    }
}

impl<V: VectorFactory> Decode<V> for Expr<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let mut instrs = V::create_vector(None);
        while reader.peek_u8()? != 0x0b {
            instrs.push(Instr::decode(reader)?);
        }
        reader.read_u8()?;
        Ok(Self { instrs })
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
pub struct Memarg {
    pub align: u32,
    pub offset: u32,
}

impl<V: VectorFactory> Decode<V> for Memarg {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let align = reader.read_u32()?;
        let offset = reader.read_u32()?;
        Ok(Self { align, offset })
    }
}

pub struct Elem<V: VectorFactory> {
    pub table: Tableidx,
    pub offset: I32ConstantExpr,
    pub init: V::Vector<Funcidx>,
}

impl<V: VectorFactory> Decode<V> for Elem<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let table = Decode::<V>::decode(reader)?;
        let offset = Decode::<V>::decode(reader)?;
        let init = Decode::<V>::decode_vector(reader)?;
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

pub(crate) struct Code<V: VectorFactory> {
    pub locals: V::Vector<Valtype>,
    pub body: Expr<V>,
}

impl<V: VectorFactory> Decode<V> for Code<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let code_size = reader.read_usize()?;
        let mut reader = Reader::new(reader.read(code_size)?);
        let mut locals = V::create_vector(None);
        let locals_len = reader.read_usize()?;
        for _ in 0..locals_len {
            let val_types_len = reader.read_usize()?;
            let val_type = Decode::<V>::decode(&mut reader)?;
            for _ in 0..val_types_len {
                locals.push(val_type);
            }
        }
        let body = Expr::decode(&mut reader)?;
        Ok(Self { locals, body })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Blocktype {
    Empty,
    Val(Valtype),
}

impl Blocktype {
    pub fn arity(self) -> usize {
        match self {
            Blocktype::Empty => 0,
            Blocktype::Val(_) => 1,
        }
    }
}

impl<V: VectorFactory> Decode<V> for Blocktype {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        if reader.read_u8()? == 0x40 {
            return Ok(Self::Empty);
        }
        reader.unread_u8();

        let t = Decode::<V>::decode(reader)?;
        Ok(Self::Val(t))
    }
}

pub struct Data<V: VectorFactory> {
    pub data: Memidx,
    pub offset: I32ConstantExpr,
    pub init: V::Vector<u8>,
}

impl<V: VectorFactory> Decode<V> for Data<V> {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let data = Decode::<V>::decode(reader)?;
        let offset = Decode::<V>::decode(reader)?;
        let init = Decode::<V>::decode_vector(reader)?;
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
