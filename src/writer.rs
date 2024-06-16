use crate::DecodeError;

#[derive(Debug)]
pub enum Writer<'a> {
    Null { position: usize },
    Data { data: &'a mut [u8], position: usize },
}

impl<'a> Writer<'a> {
    pub fn null() -> Self {
        Self::Null { position: 0 }
    }

    pub fn position(&self) -> usize {
        match self {
            Self::Null { position } => *position,
            Self::Data { position, .. } => *position,
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), DecodeError> {
        match self {
            Self::Null { position } => {
                *position += buf.len();
            }
            Self::Data { data, position } => {
                if *position + buf.len() > data.len() {
                    return Err(DecodeError::FullBytes);
                }
                data[*position..*position + buf.len()].copy_from_slice(buf);
                *position += buf.len();
            }
        }
        Ok(())
    }
}
