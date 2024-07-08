use crate::{
    decode::Decode,
    reader::Reader,
    symbols::{
        Code, Data, Elem, Export, FuncIdx, Global, Import, Magic, MemType, TableType, Version,
    },
    validation::ValidateError,
    Allocator, DecodeError, FuncType,
};

// TODO: #[derive(Debug)]
pub struct Module<A: Allocator> {
    function_types: A::Vector<FuncType<A>>,
    imports: A::Vector<Import<A>>,
    functions: A::Vector<FuncIdx>,
    table_types: A::Vector<TableType>,
    memory_type: Option<MemType>,
    globals: A::Vector<Global<A>>,
    exports: A::Vector<Export<A>>,
    start_function: Option<FuncIdx>,
    elements: A::Vector<Elem<A>>,
    function_codes: A::Vector<Code<A>>,
    data_segments: A::Vector<Data<A>>,
}

impl<A: Allocator> Module<A> {
    pub fn function_types(&self) -> &[FuncType<A>] {
        self.function_types.as_ref()
    }

    pub fn imports(&self) -> &[Import<A>] {
        self.imports.as_ref()
    }

    pub fn functions(&self) -> &[FuncIdx] {
        self.functions.as_ref()
    }

    pub fn table_types(&self) -> &[TableType] {
        self.table_types.as_ref()
    }

    pub fn memory_type(&self) -> Option<MemType> {
        self.memory_type
    }

    pub fn globals(&self) -> &[Global<A>] {
        self.globals.as_ref()
    }

    pub fn exports(&self) -> &[Export<A>] {
        self.exports.as_ref()
    }

    pub fn start_function(&self) -> Option<FuncIdx> {
        self.start_function
    }

    pub fn elements(&self) -> &[Elem<A>] {
        self.elements.as_ref()
    }

    pub fn function_codes(&self) -> &[Code<A>] {
        self.function_codes.as_ref()
    }

    pub fn data_segments(&self) -> &[Data<A>] {
        self.data_segments.as_ref()
    }

    pub fn decode(wasm_bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut this = Self {
            function_types: A::allocate_vector(),
            imports: A::allocate_vector(),
            functions: A::allocate_vector(),
            table_types: A::allocate_vector(),
            memory_type: None,
            globals: A::allocate_vector(),
            exports: A::allocate_vector(),
            start_function: None,
            elements: A::allocate_vector(),
            function_codes: A::allocate_vector(),
            data_segments: A::allocate_vector(),
        };
        let mut reader = Reader::new(wasm_bytes);

        // Preamble
        let _ = Magic::decode(&mut reader)?;
        let _ = Version::decode(&mut reader)?;

        // Sections
        this.decode_sections(&mut reader)?;

        Ok(this)
    }

    fn decode_sections(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        let mut last_section_id = SectionId::Custom;
        while !reader.is_empty() {
            let section_id = SectionId::decode(reader)?;
            let section_size = reader.read_u32()? as usize;
            let mut section_reader = Reader::new(reader.read(section_size)?);

            if section_id == SectionId::Custom {
                continue;
            }

            if section_id < last_section_id {
                return Err(DecodeError::InvalidSectionOrder {
                    current_section_id: section_id,
                    last_section_id,
                });
            }

            match section_id {
                SectionId::Custom => unreachable!(),
                SectionId::Type => {
                    self.function_types = Decode::decode_vector::<A>(&mut section_reader)?;
                }
                SectionId::Import => {
                    self.imports = Decode::decode_vector::<A>(&mut section_reader)?;
                }
                SectionId::Function => {
                    self.functions = Decode::decode_vector::<A>(&mut section_reader)?;
                }
                SectionId::Table => {
                    self.table_types = Decode::decode_vector::<A>(&mut section_reader)?;
                }
                SectionId::Memory => {
                    let value = section_reader.read_u32()? as usize;
                    if value > 1 {
                        return Err(DecodeError::InvalidMemoryCount { value });
                    }
                    if value == 1 {
                        let mem = MemType::decode(&mut section_reader)?;
                        self.memory_type = Some(mem);
                    }
                }
                SectionId::Global => {
                    self.globals = Decode::decode_vector::<A>(&mut section_reader)?;
                }
                SectionId::Export => {
                    self.exports = Decode::decode_vector::<A>(&mut section_reader)?;
                }
                SectionId::Start => {
                    self.start_function = Some(Decode::decode(&mut section_reader)?);
                }
                SectionId::Element => {
                    self.elements = Decode::decode_vector::<A>(&mut section_reader)?;
                }
                SectionId::Code => {
                    self.function_codes = Decode::decode_vector::<A>(&mut section_reader)?;
                }
                SectionId::Data => {
                    self.data_segments = Decode::decode_vector::<A>(&mut section_reader)?;
                }
            }
            last_section_id = section_id;

            if !section_reader.is_empty() {
                return Err(DecodeError::InvalidSectionSize {
                    section_id,
                    expected_size: section_size,
                    actual_size: section_reader.position(),
                });
            }
        }
        Ok(())
    }

    pub fn instantiate(&self) -> Result<(), ValidateError> {
        // TODO
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StdAllocator;

    fn decode(wasm: &[u8]) -> Module<StdAllocator> {
        Module::decode(wasm).expect("decode module")
    }

    #[test]
    fn decode_empty_module() {
        // (module)
        let input = [0, 97, 115, 109, 1, 0, 0, 0];
        decode(&input);
    }

    #[test]
    fn decode_add_two() {
        // (module
        //   (func (export "addTwo") (param i32 i32) (result i32)
        //     local.get 0
        //     local.get 1
        //     i32.add))
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 7, 1, 96, 2, 127, 127, 1, 127, 3, 2, 1, 0, 7, 10, 1, 6,
            97, 100, 100, 84, 119, 111, 0, 0, 10, 9, 1, 7, 0, 32, 0, 32, 1, 106, 11,
        ];
        let module = decode(&input);
        assert_eq!(1, module.exports().len());
        assert_eq!("addTwo", module.exports()[0].name.as_str());
    }
}
