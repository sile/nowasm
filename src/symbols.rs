use crate::reader::Reader;
use crate::writer::Writer;
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
    start: usize,
    len: usize,
}

impl Name {
    pub fn decode(reader: &mut Reader, writer: &mut Writer) -> Result<Self, DecodeError> {
        let start = writer.position();
        let len = reader.read_usize()?;
        let name = reader.read(len)?;
        let _ = core::str::from_utf8(name).map_err(DecodeError::InvalidUtf8)?;
        writer.write(name)?;
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
    pub fn decode(reader: &mut Reader, writer: &mut Writer) -> Result<Self, DecodeError> {
        let module = Name::decode(reader, writer)?;
        let name = Name::decode(reader, writer)?;
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

#[derive(Debug, Clone, Copy)]
pub struct TypeIdx(u32);

impl TypeIdx {
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
            value => Err(DecodeError::InvalidLimitsTag { value }),
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
