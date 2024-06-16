use crate::DecodeError;

#[derive(Debug)]
pub enum Writer<'a, T = u8> {
    Null { position: usize },
    Data { data: &'a mut [T], position: usize },
}

impl<'a, T: Copy> Writer<'a, T> {
    pub fn null() -> Self {
        Self::Null { position: 0 }
    }

    pub fn position(&self) -> usize {
        match self {
            Self::Null { position } => *position,
            Self::Data { position, .. } => *position,
        }
    }

    pub fn write(&mut self, buf: &[T]) -> Result<(), DecodeError> {
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
