pub trait Allocator {}

#[derive(Debug)]
pub enum DecodeError {
    EndOfBytes,
}

#[derive(Debug)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        if let Some(b) = self.data.get(self.position).copied() {
            Ok(b)
        } else {
            Err(DecodeError::EndOfBytes)
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes([
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
        ]))
    }

    pub fn section_reader(&mut self) -> Result<(u8, Self), DecodeError> {
        let section_id = self.read_u8()?;
        let size = self.read_u32()? as usize;

        let reader = Self::new(&self.data[self.position..][..size]);
        self.position += size;

        Ok((section_id, reader))
    }
}

#[derive(Debug)]
pub struct Module<A> {
    allocator: A,
}

impl<A: Allocator> Module<A> {
    pub fn decode(wasm: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = ByteReader::new(wasm);
        todo!()
    }
}
