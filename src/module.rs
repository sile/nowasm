use crate::{
    decode::Decode,
    reader::Reader,
    sections::{
        CodeSection, DataSection, ElementSection, ExportSection, FunctionSection, GlobalSection,
        ImportSection, MemorySection, SectionId, StartSection, TableSection,
    },
    symbols::{Export, Magic, Version},
    validation::ValidateError,
    Allocator, DecodeError, FuncType,
};
use std::marker::PhantomData;

// TODO: #[derive(Debug)]
pub struct Module<A: Allocator> {
    _allocator: PhantomData<A>,
    func_types: A::Vector<FuncType<A>>,
    import_section: ImportSection<A>,
    function_section: FunctionSection<A>,
    table_section: TableSection<A>,
    memory_section: MemorySection,
    global_section: GlobalSection<A>,
    export_section: ExportSection<A>,
    start_section: StartSection,
    element_section: ElementSection<A>,
    code_section: CodeSection<A>,
    data_section: DataSection<A>,
}

impl<A: Allocator> Module<A> {
    pub fn func_types(&self) -> &[FuncType<A>] {
        self.func_types.as_ref()
    }

    pub fn import_section(&self) -> &ImportSection<A> {
        &self.import_section
    }

    pub fn function_section(&self) -> &FunctionSection<A> {
        &self.function_section
    }

    pub fn table_section(&self) -> &TableSection<A> {
        &self.table_section
    }

    pub fn memory_section(&self) -> &MemorySection {
        &self.memory_section
    }

    pub fn global_section(&self) -> &GlobalSection<A> {
        &self.global_section
    }

    pub fn export_section(&self) -> &ExportSection<A> {
        &self.export_section
    }

    pub fn start_section(&self) -> &StartSection {
        &self.start_section
    }

    pub fn element_section(&self) -> &ElementSection<A> {
        &self.element_section
    }

    pub fn code_section(&self) -> &CodeSection<A> {
        &self.code_section
    }

    pub fn data_section(&self) -> &DataSection<A> {
        &self.data_section
    }

    pub fn exports(&self) -> &[Export<A>] {
        self.export_section.exports.as_ref()
    }

    pub fn decode(wasm_bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut this = Self {
            _allocator: PhantomData,
            func_types: A::allocate_vector(),
            import_section: ImportSection::new(),
            function_section: FunctionSection::new(),
            table_section: TableSection::new(),
            memory_section: MemorySection::default(),
            global_section: GlobalSection::new(),
            export_section: ExportSection::new(),
            start_section: StartSection::default(),
            element_section: ElementSection::new(),
            code_section: CodeSection::new(),
            data_section: DataSection::new(),
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
                    self.func_types = FuncType::decode_vector::<A>(reader)?;
                }
                SectionId::Import => {
                    self.import_section = ImportSection::decode(&mut section_reader)?
                }
                SectionId::Function => {
                    self.function_section = FunctionSection::decode(&mut section_reader)?
                }
                SectionId::Table => self.table_section = TableSection::decode(&mut section_reader)?,
                SectionId::Memory => {
                    self.memory_section = MemorySection::decode(&mut section_reader)?
                }
                SectionId::Global => {
                    self.global_section = GlobalSection::decode(&mut section_reader)?
                }
                SectionId::Export => {
                    self.export_section = ExportSection::decode(&mut section_reader)?
                }
                SectionId::Start => self.start_section = StartSection::decode(&mut section_reader)?,
                SectionId::Element => {
                    self.element_section = ElementSection::decode(&mut section_reader)?
                }
                SectionId::Code => self.code_section = CodeSection::decode(&mut section_reader)?,
                SectionId::Data => self.data_section = DataSection::decode(&mut section_reader)?,
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
