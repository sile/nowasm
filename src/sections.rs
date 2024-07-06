use crate::{
    reader::Reader,
    symbols::{
        Code, Data, Elem, Export, FuncIdx, FuncType, Global, Import, MemType, TableType, TypeIdx,
    },
    Allocator, DecodeError, DecodeVector,
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

// TODO: derive debug
pub struct TypeSection<A: Allocator> {
    pub types: A::Vector<FuncType<A>>,
}

impl<A: Allocator> TypeSection<A> {
    pub(crate) fn new() -> Self {
        Self {
            types: A::allocate_vector(),
        }
    }

    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let types = FuncType::<A>::decode_vector(reader)?;
        Ok(Self { types })
    }
}

#[derive(Debug, Clone)]
pub struct ImportSection<A: Allocator> {
    pub imports: A::Vector<Import<A>>,
}

impl<A: Allocator> ImportSection<A> {
    pub(crate) fn new() -> Self {
        Self {
            imports: A::allocate_vector(),
        }
    }

    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let imports = DecodeVector::<A>::decode_vector(reader)?;
        Ok(Self { imports })
    }
}

#[derive(Debug, Clone)]
pub struct FunctionSection<A: Allocator> {
    pub idxs: A::Vector<TypeIdx>,
}

impl<A: Allocator> FunctionSection<A> {
    pub(crate) fn new() -> Self {
        Self {
            idxs: A::allocate_vector(),
        }
    }

    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let idxs = DecodeVector::<A>::decode_vector(reader)?;
        Ok(Self { idxs })
    }
}

#[derive(Debug, Clone)]
pub struct TableSection<A: Allocator> {
    pub tables: A::Vector<TableType>,
}

impl<A: Allocator> TableSection<A> {
    pub(crate) fn new() -> Self {
        Self {
            tables: A::allocate_vector(),
        }
    }

    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let tables = DecodeVector::<A>::decode_vector(reader)?;
        Ok(Self { tables })
    }
}

#[derive(Debug, Default, Clone)]
pub struct MemorySection {
    pub mem: Option<MemType>,
}

impl MemorySection {
    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let value = reader.read_u32()? as usize;
        if value > 1 {
            return Err(DecodeError::InvalidMemoryCount { value });
        }
        if value == 0 {
            Ok(Self { mem: None })
        } else {
            let mem = MemType::decode(reader)?;
            Ok(Self { mem: Some(mem) })
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalSection<A: Allocator> {
    pub globals: A::Vector<Global<A>>,
}

impl<A: Allocator> GlobalSection<A> {
    pub(crate) fn new() -> Self {
        Self {
            globals: A::allocate_vector(),
        }
    }

    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let globals = DecodeVector::<A>::decode_vector(reader)?;
        Ok(Self { globals })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ExportSection<A: Allocator> {
    pub exports: A::Vector<Export<A>>,
}

impl<A: Allocator> ExportSection<A> {
    pub(crate) fn new() -> Self {
        Self {
            exports: A::allocate_vector(),
        }
    }

    pub(crate) fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let exports = DecodeVector::<A>::decode_vector(reader)?;
        Ok(Self { exports })
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
        let elems = DecodeVector::<A>::decode_vector(reader)?;
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
        let codes = DecodeVector::<A>::decode_vector(reader)?;
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
        let datas = DecodeVector::<A>::decode_vector(reader)?;
        Ok(Self { datas })
    }
}
