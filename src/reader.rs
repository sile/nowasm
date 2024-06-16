use crate::DecodeError;

#[derive(Debug)]
pub struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data, position: 0 }
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        let v = self
            .data
            .get(self.position)
            .copied()
            .ok_or(DecodeError::EndOfBytes)?;
        self.position += 1;
        Ok(v)
    }

    pub fn read(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        let v = self
            .data
            .get(self.position..self.position + n)
            .ok_or(DecodeError::EndOfBytes)?;
        self.position += n;
        Ok(v)
    }
}
