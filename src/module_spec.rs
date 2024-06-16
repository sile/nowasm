use crate::{
    reader::Reader,
    symbols::{Magic, SectionId, Version},
    DecodeError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleSpec {}

impl ModuleSpec {
    pub fn inspect(wasm_bytes: &[u8]) -> Result<ModuleSpec, DecodeError> {
        let mut reader = Reader::new(wasm_bytes);
        let mut this = Self {};
        this.handle_module(&mut reader)?;
        Ok(this)
    }

    fn handle_module(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        // Preamble
        let _ = Magic::decode(reader)?;
        let _ = Version::decode(reader)?;

        // Sections
        while !reader.is_empty() {
            let section_id = SectionId::decode(reader)?;
            let section_size = reader.read_u32()? as usize;
            let mut section_reader = Reader::new(reader.read(section_size)?);
            match section_id {
                SectionId::Custom => {}
                SectionId::Type => todo!(),
                SectionId::Import => todo!(),
                SectionId::Function => todo!(),
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
}
