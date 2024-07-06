use crate::{
    reader::Reader,
    sections::{
        CodeSection, DataSection, ElementSection, ExportSection, FunctionSection, GlobalSection,
        ImportSection, MemorySection, SectionId, StartSection, TableSection, TypeSection,
    },
    symbols::{Export, Magic, Version},
    validation::ValidateError,
    Allocator, DecodeError,
};
use std::marker::PhantomData;

// TODO: #[derive(Debug)]
pub struct Module<A: Allocator> {
    _allocator: PhantomData<A>,
    type_section: TypeSection<A>,
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
    pub fn type_section(&self) -> &TypeSection<A> {
        &self.type_section
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

    pub fn exports(&self) -> impl '_ + Iterator<Item = &Export<A>> {
        self.export_section.exports.as_ref().iter()
    }

    pub fn decode(wasm_bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut this = Self {
            _allocator: PhantomData,
            type_section: TypeSection::new(),
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
                SectionId::Type => self.type_section = TypeSection::decode(&mut section_reader)?,
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

    pub fn validate(&self) -> Result<(), ValidateError> {
        // TODO
        Ok(())
    }
}
