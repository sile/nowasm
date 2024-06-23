use crate::{
    reader::Reader,
    sections::{ImportSection, SectionId, TypeSection},
    symbols::{Magic, Version},
    DecodeError, Vectors,
};

#[derive(Debug, Clone)]
pub struct Module<V> {
    vectors: V,
    type_section: TypeSection,
    import_section: ImportSection,
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

    pub fn decode(wasm_bytes: &[u8], vectors: V) -> Result<Self, DecodeError> {
        let mut this = Self {
            vectors,
            type_section: TypeSection::default(),
            import_section: ImportSection::default(),
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
        while reader.is_empty() {
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
                _ => todo!(),
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
}
