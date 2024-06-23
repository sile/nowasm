use crate::{
    decode::Decode,
    instructions::Instr,
    reader::Reader,
    symbols::{Locals, ValType},
    DecodeError,
};
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct VectorSlice<T> {
    offset: usize,
    len: usize,
    _item: PhantomData<T>,
}

impl<T> VectorSlice<T> {
    pub fn len(self) -> usize {
        self.len
    }

    pub fn is_empty(self) -> bool {
        self.len == 0
    }
}

impl<T: VectorItem> Decode for VectorSlice<T> {
    fn decode<V: Vectors>(reader: &mut Reader, vectors: &mut V) -> Result<Self, DecodeError> {
        let offset = T::append(vectors, &[])?;
        let len = reader.read_usize()?;
        for _ in 0..len {
            let item = T::decode(reader, vectors)?;
            T::append(vectors, &[item])?;
        }
        Ok(Self {
            offset,
            len,
            _item: PhantomData,
        })
    }
}

pub trait VectorItem: Decode {
    fn get<V: Vectors>(vectors: &mut V) -> Option<Self>;
    fn append<V: Vectors>(vectors: &mut V, items: &[Self]) -> Result<usize, DecodeError>;
    // TODO: Add decode()
}

pub trait Vectors {
    fn bytes_offset(&self) -> usize; // TODO: bytes(&self) -> &[u8]
    fn bytes_append(&mut self, bytes: &[u8]) -> bool;

    fn val_types_offset(&self) -> usize;
    fn val_types_push(&mut self, val_type: ValType) -> bool;

    fn instrs_offset(&self) -> usize;
    fn instrs_push(&mut self, instr: Instr) -> bool;

    fn idxs_offset(&self) -> usize;
    fn idxs_push(&mut self, idx: u32) -> bool;

    fn locals_offset(&self) -> usize;
    fn locals_push(&mut self, locals: Locals) -> bool;
}

#[derive(Debug, Default)]
pub struct NullVectors {
    bytes_offset: usize,
    val_types_offset: usize,
    instrs_offset: usize,
    idxs_offset: usize,
    locals_offset: usize,
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

    fn locals_offset(&self) -> usize {
        self.locals_offset
    }

    fn locals_push(&mut self, _locals: Locals) -> bool {
        self.locals_offset += 1;
        true
    }
}
