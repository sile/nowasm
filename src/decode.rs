use crate::vectors::Vector;
use crate::{reader::Reader, symbols::SectionId, Allocator};
use core::str::Utf8Error;
use std::fmt::Debug;

pub trait Decode: Sized {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError>;

    fn decode_vector<A: Allocator>(reader: &mut Reader) -> Result<A::Vector<Self>, DecodeError> {
        let len = reader.read_usize()?;
        let mut items = A::allocate_vector();
        for _ in 0..len {
            items.push(Self::decode(reader)?);
        }
        Ok(items)
    }
}

impl Decode for u8 {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_u8()
    }
}

// TODO: impl Display
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    EndOfBytes,
    InvalidMagic {
        value: [u8; 4],
    },
    InvalidVersion {
        value: [u8; 4],
    },
    InvalidSectionId {
        value: u8,
    },
    InvalidImportDescTag {
        value: u8,
    },
    InvalidExportDescTag {
        value: u8,
    },
    InvalidLimitsFlag {
        value: u8,
    },
    InvalidValType {
        value: u8,
    },
    InvalidMutabilityFlag {
        value: u8,
    },
    InvalidElemType {
        value: u8,
    },
    InvalidFuncTypeTag {
        value: u8,
    },
    InvalidMemoryCount {
        value: usize,
    },
    InvalidMemIdx {
        value: u32,
    },
    InvalidTableIdx {
        value: u32,
    },
    InvalidMemorySizeMemoryIndex {
        value: u8,
    },
    InvalidMemoryGrowMemoryIndex {
        value: u8,
    },
    InvalidCallIndirectTableIndex {
        value: u8,
    },
    InvalidOpcode {
        value: u8,
    },
    InvalidSectionOrder {
        last_section_id: SectionId,
        current_section_id: SectionId,
    },
    InvalidSectionSize {
        section_id: SectionId,
        expected_size: usize,
        actual_size: usize,
    },
    InvalidUtf8(Utf8Error),
    MalformedInteger,
}
