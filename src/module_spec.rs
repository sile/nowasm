use crate::{
    reader::Reader,
    symbols::{Magic, Version},
    DecodeError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleSpec {}

impl ModuleSpec {
    pub fn inspect(wasm_bytes: &[u8]) -> Result<ModuleSpec, DecodeError> {
        let reader = Reader::new(wasm_bytes);
        let mut this = Self {};
        this.handle_module(reader)?;
        Ok(this)
    }

    fn handle_module(&mut self, mut reader: Reader) -> Result<(), DecodeError> {
        // Preamble
        let _ = Magic::decode(&mut reader)?;
        let _ = Version::decode(&mut reader)?;

        Ok(())
    }
}
