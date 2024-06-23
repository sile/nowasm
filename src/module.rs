use crate::{
    reader::Reader,
    sections::SectionId,
    symbols::{Magic, Version},
    DecodeError, Vectors,
};

#[derive(Debug, Clone)]
pub struct Module<V> {
    vectors: V,
}

impl<V: Vectors> Module<V> {
    pub fn vectors(&self) -> &V {
        &self.vectors
    }

    pub fn vectors_mut(&mut self) -> &mut V {
        &mut self.vectors
    }

    pub fn decode(wasm_bytes: &[u8], vectors: V) -> Result<Self, DecodeError> {
        let mut this = Self { vectors };
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
                SectionId::Type => self.decode_type_section(&mut section_reader)?,
                // 2 => self.decode_import_section(&mut section_reader)?,
                // 3 => self.decode_function_section(&mut section_reader)?,
                // 4 => self.decode_table_section(&mut section_reader)?,
                // 5 => self.decode_memory_section(&mut section_reader)?,
                // 6 => self.decode_global_section(&mut section_reader)?,
                // 7 => self.decode_export_section(&mut section_reader)?,
                // 8 => self.decode_start_section(&mut section_reader)?,
                // 9 => self.decode_element_section(&mut section_reader)?,
                // 10 => self.decode_code_section(&mut section_reader)?,
                // 11 => self.decode_data_section(&mut section_reader)?,
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

    fn decode_type_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        Ok(())
    }
}
