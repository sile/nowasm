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

#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

#[derive(Debug, Clone, Copy)]
pub struct TypeIdx(u32);

#[derive(Debug, Clone)]
pub struct TableType {
    pub limits: Limits,
}

#[derive(Debug, Clone)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct MemType {
    pub limits: Limits,
}

#[derive(Debug, Clone)]
pub enum GlobalType {
    Const(ValType),
    Var(ValType),
}

#[derive(Debug, Clone, Copy)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
}
