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

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        if let Some(b) = self.data.get(self.position).copied() {
            Ok(b)
        } else {
            Err(DecodeError::EndOfBytes)
        }
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
