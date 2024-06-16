use crate::reader::Reader;
use crate::DecodeError;

#[derive(Debug)]
pub struct Magic;

impl Magic {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        if reader.read(4)? != b"\0asm" {
            return Err(DecodeError::InvalidMagic);
        }
        Ok(Self)
    }
}

#[derive(Debug)]
pub struct Version;

impl Version {
    pub fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        if reader.read(4)? != [1, 0, 0, 0] {
            return Err(DecodeError::InvalidVersion);
        }
        Ok(Self)
    }
}
