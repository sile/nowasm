#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    EndOfBytes,
    InvalidMagic,
    InvalidVersion,
}
