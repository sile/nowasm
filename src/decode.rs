use crate::{reader::Reader, DecodeError, Vectors};

pub trait Decode: Sized {
    fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError>;
}
