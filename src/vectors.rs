use crate::{instructions::Instr, symbols::ValType};

pub trait Vectors {
    fn bytes_offset(&self) -> usize;
    fn bytes_append(&mut self, bytes: &[u8]) -> bool;

    fn val_types_offset(&self) -> usize;
    fn val_types_push(&mut self, val_type: ValType) -> bool;

    fn instrs_offset(&self) -> usize;
    fn instrs_push(&mut self, instr: Instr) -> bool;

    fn idxs_offset(&self) -> usize;
    fn idxs_push(&mut self, idx: u32) -> bool;
}

#[derive(Debug, Default)]
pub struct NullVectors {
    bytes_offset: usize,
    val_types_offset: usize,
    instrs_offset: usize,
    idxs_offset: usize,
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

    fn instrs_offset(&self) -> usize {
        self.instrs_offset
    }

    fn instrs_push(&mut self, _instr: Instr) -> bool {
        self.instrs_offset += 1;
        true
    }

    fn idxs_offset(&self) -> usize {
        self.idxs_offset
    }

    fn idxs_push(&mut self, _idx: u32) -> bool {
        self.idxs_offset += 1;
        true
    }
}
