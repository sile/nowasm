use crate::{reader::Reader, DecodeError};

#[derive(Debug, Clone, Copy)]
pub enum SignExtensionInstr {}

impl SignExtensionInstr {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let opcode = reader.read_u8()?;
        match opcode {
            _ => Err(DecodeError::InvalidOpcode { value: opcode }),
        }
    }
}
