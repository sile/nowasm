pub trait Allocator {}

#[derive(Debug)]
pub enum DecodeError {}

#[derive(Debug)]
pub struct Module<A> {
    allocator: A,
}

impl<A: Allocator> Module<A> {
    pub fn decode(wasm: &[u8]) -> Result<Self, DecodeError> {
        todo!()
    }
}
