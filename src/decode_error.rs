use crate::{symbols::SectionId, vectors::VectorKind};
use core::str::Utf8Error;

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
    FullVector {
        kind: VectorKind,
    },
}
