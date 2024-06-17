use crate::{
    reader::Reader,
    symbols::{Import, Magic, SectionId, Version},
    writer::Writer,
    DecodeError,
};

#[derive(Debug, Clone)]
pub struct ModuleSpec {
    pub types: usize,
    pub imports: usize,
    pub bytes: usize,
    pub idxs: usize,
}

impl ModuleSpec {
    pub fn inspect(wasm_bytes: &[u8]) -> Result<ModuleSpec, DecodeError> {
        let mut reader = Reader::new(wasm_bytes);
        let mut this = Self {
            types: 0,
            imports: 0,
            bytes: 0,
            idxs: 0,
        };
        this.handle_module(&mut reader)?;
        Ok(this)
    }

    fn handle_module(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        // Preamble
        let _ = Magic::decode(reader)?;
        let _ = Version::decode(reader)?;

        // Sections
        let mut last_section_id = SectionId::Custom;
        while !reader.is_empty() {
            let section_id = SectionId::decode(reader)?;
            if section_id != SectionId::Custom {
                if section_id <= last_section_id {
                    return Err(DecodeError::OutOfOrderSectionId {
                        last: last_section_id,
                        current: section_id,
                    });
                }
                last_section_id = section_id;
            }

            let section_size = reader.read_u32()? as usize;
            let mut section_reader = Reader::new(reader.read(section_size)?);

            match section_id {
                SectionId::Custom => {}
                SectionId::Type => self.handle_type_section(&mut section_reader)?,
                SectionId::Import => self.handle_import_section(&mut section_reader)?,
                SectionId::Function => self.handle_function_section(&mut section_reader)?,
                SectionId::Table => todo!(),
                SectionId::Memory => todo!(),
                SectionId::Global => todo!(),
                SectionId::Export => todo!(),
                SectionId::Start => todo!(),
                SectionId::Element => todo!(),
                SectionId::Code => todo!(),
                SectionId::Data => todo!(),
            }
        }

        Ok(())
    }

    fn handle_type_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.types = reader.read_usize()?;
        Ok(())
    }

    fn handle_import_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.imports = reader.read_usize()?;
        for _ in 0..self.imports {
            let import = Import::decode(reader, &mut Writer::null())?;
            self.bytes += import.module.len();
            self.bytes += import.name.len();
        }
        Ok(())
    }

    fn handle_function_section(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        self.idxs = reader.read_usize()?;
        Ok(())
    }
}
