use crate::vector::Vector;
use crate::{reader::Reader, VectorFactory};
use core::fmt::{Display, Formatter};
use core::str::Utf8Error;

pub trait Decode: Sized {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError>;

    fn decode_vector<V: VectorFactory>(
        reader: &mut Reader,
    ) -> Result<V::Vector<Self>, DecodeError> {
        let len = reader.read_usize()?;
        let mut items = V::allocate_vector();
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    UnexpectedEndOfBytes,
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
    InvalidOpcode {
        value: u8,
    },
    InvalidSectionOrder {
        last_section_id: u8,
        current_section_id: u8,
    },
    InvalidSectionSize {
        section_id: u8,
        expected_size: usize,
        actual_size: usize,
    },
    InvalidUtf8(Utf8Error),
    MalformedInteger,
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEndOfBytes => write!(f, "Unexpected end-of-bytes"),
            Self::InvalidMagic { value } => write!(f, "Invalid magic number {value:?}"),
            Self::InvalidVersion { value } => write!(f, "Invalid version number {value:?}"),
            Self::InvalidSectionId { value } => write!(f, "Invalid section ID {value:?}"),
            Self::InvalidImportDescTag { value } => {
                write!(f, "Invalid import description tag {value:?})")
            }
            Self::InvalidExportDescTag { value } => {
                write!(f, "Invalid export description tag {value:?})")
            }
            Self::InvalidLimitsFlag { value } => write!(f, "Invalid limits flag {value:?}"),
            Self::InvalidValType { value } => write!(f, "Invalid value type {value:?}"),
            Self::InvalidMutabilityFlag { value } => write!(f, "Invalid mutability flag {value:?}"),
            Self::InvalidElemType { value } => write!(f, "Invalid element type {value:?}"),
            Self::InvalidFuncTypeTag { value } => write!(f, "Invalid function type {value:?}"),
            Self::InvalidMemoryCount { value } => write!(f, "Invalid memory count {value:?}"),
            Self::InvalidMemIdx { value } => write!(f, "Invalid memory index {value:?}"),
            Self::InvalidTableIdx { value } => write!(f, "Invalid table index {value:?}"),
            Self::InvalidOpcode { value } => write!(f, "Invalid opcode {value:?}"),
            Self::InvalidSectionOrder {
                last_section_id,
                current_section_id,
            } => write!(
                f,
                "Invalid section order (last={last_section_id:?}), current={current_section_id:?})"
            ),
            Self::InvalidSectionSize {
                section_id,
                expected_size,
                actual_size,
            } => write!(f,"Invalid section {section_id:?} size (expected={expected_size:?} bytes, actual={actual_size:?} bytes)"),
            Self::InvalidUtf8(e) => write!(f,"Invalid UTF-8 bytes ({e})"),
            Self::MalformedInteger => write!(f,"Malformed LEB128 integer"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}
