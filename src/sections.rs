use crate::{
    decode::Decode,
    reader::Reader,
    symbols::{FuncType, Import, TableType, TypeIdx},
    DecodeError, VectorSlice, Vectors,
};

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
    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
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

#[derive(Debug, Default, Clone)]
pub struct TypeSection {
    pub types: VectorSlice<FuncType>,
}

impl TypeSection {
    pub(crate) fn decode(
        reader: &mut Reader,
        vectors: &mut impl Vectors,
    ) -> Result<Self, DecodeError> {
        let types = VectorSlice::decode(reader, vectors)?;
        Ok(Self { types })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ImportSection {
    pub imports: VectorSlice<Import>,
}

impl ImportSection {
    pub(crate) fn decode(
        reader: &mut Reader,
        vectors: &mut impl Vectors,
    ) -> Result<Self, DecodeError> {
        let imports = VectorSlice::decode(reader, vectors)?;
        Ok(Self { imports })
    }
}

#[derive(Debug, Default, Clone)]
pub struct FunctionSection {
    pub idxs: VectorSlice<TypeIdx>,
}

impl FunctionSection {
    pub(crate) fn decode(
        reader: &mut Reader,
        vectors: &mut impl Vectors,
    ) -> Result<Self, DecodeError> {
        let idxs = VectorSlice::decode(reader, vectors)?;
        Ok(Self { idxs })
    }
}

#[derive(Debug, Default, Clone)]
pub struct TableSection {
    pub tables: VectorSlice<TableType>,
}

impl TableSection {
    pub(crate) fn decode(
        reader: &mut Reader,
        vectors: &mut impl Vectors,
    ) -> Result<Self, DecodeError> {
        let tables = VectorSlice::decode(reader, vectors)?;
        Ok(Self { tables })
    }
}
