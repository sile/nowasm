#[derive(Debug)]
pub struct Module {}

impl Module {
    pub fn decode(wasm: &[u8]) -> Result<Module, DecodeError> {
        Ok(Module {})
    }
}

#[derive(Debug)]
pub enum DecodeError {}
