use crate::{
    reader::Reader,
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
        while reader.is_empty() {}
        Ok(())
    }
}
