use crate::instructions::Instr;
use crate::reader::Reader;
use crate::vectors::Vectors;
use crate::DecodeError;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SectionId {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
}

impl SectionId {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            0 => Ok(Self::Custom),
            1 => Ok(Self::Type),
            2 => Ok(Self::Import),
            3 => Ok(Self::Function),
            4 => Ok(Self::Table),
            5 => Ok(Self::Memory),
            6 => Ok(Self::Global),
            7 => Ok(Self::Export),
            8 => Ok(Self::Start),
            9 => Ok(Self::Element),
            10 => Ok(Self::Code),
            11 => Ok(Self::Data),
            value => Err(DecodeError::InvalidSectionId { value }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Name {
    pub start: usize, // TODO: priv
    len: usize,
}

impl Name {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let start = vectors.bytes_offset();
        let len = reader.read_usize()?;
        let name = reader.read(len)?;
        let _ = core::str::from_utf8(name).map_err(DecodeError::InvalidUtf8)?;
        if !vectors.bytes_append(name) {
            return Err(DecodeError::FullBytes);
        }
        Ok(Self { start, len })
    }

    pub fn len(self) -> usize {
        self.len
    }
}

#[derive(Debug, Clone)]
pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

impl Import {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let module = Name::decode(reader, vectors)?;
        let name = Name::decode(reader, vectors)?;
        let desc = ImportDesc::decode(reader)?;
        Ok(Self { module, name, desc })
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Export {
    pub name: Name,
    pub desc: ExportDesc,
}

impl Export {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let name = Name::decode(reader, vectors)?;
        let desc = ExportDesc::decode(reader)?;
        Ok(Self { name, desc })
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
pub struct TypeIdx(u32);

impl TypeIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FuncIdx(u32);

impl FuncIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TableIdx(u32);

impl TableIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemIdx(u32);

impl MemIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalIdx(u32);

impl GlobalIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocalIdx(u32);

impl LocalIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LabelIdx(u32);

impl LabelIdx {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u32().map(Self)
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct FuncType {
    pub rt1: ResultType,
    pub rt2: ResultType,
}

impl FuncType {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        if tag != 0x60 {
            return Err(DecodeError::InvalidFuncTypeTag { value: tag });
        }
        let rt1 = ResultType::decode(reader, vectors)?;
        let rt2 = ResultType::decode(reader, vectors)?;
        Ok(Self { rt1, rt2 })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResultType {
    pub start: usize, // TODO: priv
    len: usize,
}

impl ResultType {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let start = vectors.val_types_offset();
        let len = reader.read_usize()?;
        for _ in 0..len {
            let vt = ValType::decode(reader)?;
            if !vectors.val_types_push(vt) {
                return Err(DecodeError::FullValTypes);
            }
        }
        Ok(Self { start, len })
    }

    pub fn len(self) -> usize {
        self.len
    }
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone, Copy)]
pub struct Expr {
    start: usize,
    len: usize,
}

impl Expr {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let start = vectors.instrs_offset();
        while reader.peek_u8()? != 0x0b {
            let instr = Instr::decode(reader, vectors)?;
            if !vectors.instrs_push(instr) {
                return Err(DecodeError::FullInstrs);
            }
        }
        reader.read_u8()?;
        let end = vectors.instrs_offset();
        let len = end - start;
        Ok(Self { start, len })
    }

    pub fn len(self) -> usize {
        self.len
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
pub struct FuncIdxVec {
    pub start: usize,
    len: usize,
}

impl FuncIdxVec {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let start = vectors.idxs_offset();
        let len = reader.read_usize()?;
        for _ in 0..len {
            let idx = FuncIdx::decode(reader)?;
            if !vectors.idxs_push(idx.0) {
                return Err(DecodeError::FullIdxs);
            }
        }
        Ok(Self { start, len })
    }

    pub fn len(self) -> usize {
        self.len
    }
}

#[derive(Debug, Clone)]
pub struct Code {}

impl Code {
    pub fn decode(reader: &mut Reader, vectors: &mut impl Vectors) -> Result<Self, DecodeError> {
        let code_size = reader.read_usize()?;
        let mut reader = Reader::new(reader.read(code_size)?);
        todo!()
    }
}
