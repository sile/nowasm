use crate::symbols::ValType;

pub trait Vectors {
    fn bytes_offset(&self) -> usize;
    fn bytes_append(&mut self, bytes: &[u8]) -> bool;

    fn val_types_offset(&self) -> usize;
    fn val_types_push(&mut self, val_type: ValType) -> bool;
}

#[derive(Debug, Default)]
pub struct NullVectors {
    bytes_offset: usize,
    val_types_offset: usize,
}

impl Vectors for NullVectors {
    fn bytes_offset(&self) -> usize {
        self.bytes_offset
    }

    fn bytes_append(&mut self, bytes: &[u8]) -> bool {
        self.bytes_offset += bytes.len();
        true
    }

    fn val_types_offset(&self) -> usize {
        self.val_types_offset
    }

    fn val_types_push(&mut self, _val_type: ValType) -> bool {
        self.val_types_offset += 1;
        true
    }
}
