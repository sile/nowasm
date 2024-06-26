use crate::{
    reader::Reader,
    sections::{
        CodeSection, DataSection, ElementSection, ExportSection, FunctionSection, GlobalSection,
        ImportSection, MemorySection, SectionId, StartSection, TableSection, TypeSection,
    },
    symbols::{Magic, Version},
    validation::ValidateError,
    DecodeError, Vectors,
};

#[derive(Debug, Clone)]
pub struct Module<V> {
    vectors: V,
    type_section: TypeSection,
    import_section: ImportSection,
    function_section: FunctionSection,
    table_section: TableSection,
    memory_section: MemorySection,
    global_section: GlobalSection,
    export_section: ExportSection,
    start_section: StartSection,
    element_section: ElementSection,
    code_section: CodeSection,
    data_section: DataSection,
}

impl<V: Vectors> Module<V> {
    pub fn vectors(&self) -> &V {
        &self.vectors
    }

    pub fn type_section(&self) -> &TypeSection {
        &self.type_section
    }

    pub fn import_section(&self) -> &ImportSection {
        &self.import_section
    }

    pub fn function_section(&self) -> &FunctionSection {
        &self.function_section
    }

    pub fn table_section(&self) -> &TableSection {
        &self.table_section
    }

    pub fn memory_section(&self) -> &MemorySection {
        &self.memory_section
    }

    pub fn global_section(&self) -> &GlobalSection {
        &self.global_section
    }

    pub fn export_section(&self) -> &ExportSection {
        &self.export_section
    }

    pub fn start_section(&self) -> &StartSection {
        &self.start_section
    }

    pub fn element_section(&self) -> &ElementSection {
        &self.element_section
    }

    pub fn code_section(&self) -> &CodeSection {
        &self.code_section
    }

    pub fn data_section(&self) -> &DataSection {
        &self.data_section
    }

    pub fn decode(wasm_bytes: &[u8], vectors: V) -> Result<Self, DecodeError> {
        let mut this = Self {
            vectors,
            type_section: TypeSection::default(),
            import_section: ImportSection::default(),
            function_section: FunctionSection::default(),
            table_section: TableSection::default(),
            memory_section: MemorySection::default(),
            global_section: GlobalSection::default(),
            export_section: ExportSection::default(),
            start_section: StartSection::default(),
            element_section: ElementSection::default(),
            code_section: CodeSection::default(),
            data_section: DataSection::default(),
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
                    self.type_section = TypeSection::decode(&mut section_reader, &mut self.vectors)?
                }
                SectionId::Import => {
                    self.import_section =
                        ImportSection::decode(&mut section_reader, &mut self.vectors)?
                }
                SectionId::Function => {
                    self.function_section =
                        FunctionSection::decode(&mut section_reader, &mut self.vectors)?
                }
                SectionId::Table => {
                    self.table_section =
                        TableSection::decode(&mut section_reader, &mut self.vectors)?
                }
                SectionId::Memory => {
                    self.memory_section = MemorySection::decode(&mut section_reader)?
                }
                SectionId::Global => {
                    self.global_section =
                        GlobalSection::decode(&mut section_reader, &mut self.vectors)?
                }
                SectionId::Export => {
                    self.export_section =
                        ExportSection::decode(&mut section_reader, &mut self.vectors)?
                }
                SectionId::Start => self.start_section = StartSection::decode(&mut section_reader)?,
                SectionId::Element => {
                    self.element_section =
                        ElementSection::decode(&mut section_reader, &mut self.vectors)?
                }
                SectionId::Code => {
                    self.code_section = CodeSection::decode(&mut section_reader, &mut self.vectors)?
                }
                SectionId::Data => {
                    self.data_section = DataSection::decode(&mut section_reader, &mut self.vectors)?
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

    pub fn validate(&self) -> Result<(), ValidateError> {
        // TODO
        Ok(())
    }
}
