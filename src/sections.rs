use crate::{
    decode::Decode,
    reader::Reader,
    symbols::{Code, Data, Elem, FuncIdx},
    Allocator, DecodeError,
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
pub struct StartSection {
    pub start: Option<FuncIdx>,
}

impl StartSection {
    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let start = FuncIdx::decode(reader)?;
        Ok(Self { start: Some(start) })
    }
}

#[derive(Debug, Clone)]
pub struct ElementSection<A: Allocator> {
    pub elems: A::Vector<Elem<A>>,
}

impl<A: Allocator> ElementSection<A> {
    pub(crate) fn new() -> Self {
        Self {
            elems: A::allocate_vector(),
        }
    }

    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let elems = Decode::decode_vector::<A>(reader)?;
        Ok(Self { elems })
    }
}

#[derive(Debug, Clone)]
pub struct CodeSection<A: Allocator> {
    pub codes: A::Vector<Code<A>>,
}

impl<A: Allocator> CodeSection<A> {
    pub(crate) fn new() -> Self {
        Self {
            codes: A::allocate_vector(),
        }
    }

    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let codes = Decode::decode_vector::<A>(reader)?;
        Ok(Self { codes })
    }
}

#[derive(Debug, Clone)]
pub struct DataSection<A: Allocator> {
    pub datas: A::Vector<Data<A>>,
}

impl<A: Allocator> DataSection<A> {
    pub(crate) fn new() -> Self {
        Self {
            datas: A::allocate_vector(),
        }
    }

    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let datas = Decode::decode_vector::<A>(reader)?;
        Ok(Self { datas })
    }
}
