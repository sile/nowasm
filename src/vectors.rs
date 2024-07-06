use crate::{reader::Reader, DecodeError};
use core::fmt::Debug;

// TODO: Remove Debug and Clone bound
pub trait Allocator: Debug + Clone {
    type Vector<T: Clone>: Vector<T>;

    fn allocate_vector<T: Clone>() -> Self::Vector<T>;
}

// TODO: Remove Debug and Clone bound
pub trait Vector<T: Clone>: Debug + Clone + AsRef<[T]> + AsMut<[T]> {
    fn push(&mut self, item: T);
}

// TODO: rename
pub trait DecodeVector<A: Allocator>: Sized + Clone {
    fn decode_item(reader: &mut Reader) -> Result<Self, DecodeError>;
    fn decode_vector(reader: &mut Reader) -> Result<A::Vector<Self>, DecodeError> {
        let len = reader.read_usize()?;
        let mut vs = A::allocate_vector();
        for _ in 0..len {
            let item = Self::decode_item(reader)?;
            vs.push(item);
        }
        Ok(vs)
    }
}
