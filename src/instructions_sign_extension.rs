use crate::{reader::Reader, DecodeError};

#[derive(Debug, Clone, Copy)]
pub enum SignExtensionInstr {
    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,
}

impl SignExtensionInstr {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let opcode = reader.read_u8()?;
        match opcode {
            0xc0 => Ok(SignExtensionInstr::I32Extend8S),
            0xc1 => Ok(SignExtensionInstr::I32Extend16S),
            0xc2 => Ok(SignExtensionInstr::I64Extend8S),
            0xc3 => Ok(SignExtensionInstr::I64Extend16S),
            0xc4 => Ok(SignExtensionInstr::I64Extend32S),
            _ => Err(DecodeError::InvalidOpcode { value: opcode }),
        }
    }
}
