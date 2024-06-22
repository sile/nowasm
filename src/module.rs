use crate::{DecodeError, Vectors};

#[derive(Debug, Clone)]
pub struct Module<V> {
    vectors: V,
}

impl<V: Vectors> Module<V> {
    pub fn new(vectors: V) -> Self {
        Self { vectors }
    }

    pub fn vectors(&self) -> &V {
        &self.vectors
    }

    pub fn vectors_mut(&mut self) -> &mut V {
        &mut self.vectors
    }

    pub fn decode(wasm_bytes: &[u8]) -> Result<Self, DecodeError> {
        todo!()
    }
}
