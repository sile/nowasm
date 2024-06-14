pub trait Allocator {}

#[derive(Debug, Clone, Copy)]
pub enum DecodeError {
    EndOfBytes,
    InvalidMagicNumber,
    UnsupportedVersion,
    InvalidSectionId { value: u8 },
    TooLargeSectionSize { section_id: SectionId, size: usize },
    InvalidU32,
}

#[derive(Debug)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        if let Some(b) = self.data.get(self.position).copied() {
            self.position += 1;
            Ok(b)
        } else {
            Err(DecodeError::EndOfBytes)
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, DecodeError> {
        let mut n = 0u32;
        let mut bits = 0;
        loop {
            let b = self.read_u8()?;
            n = n
                .checked_add((b as u32 & 0b0111_1111) << bits)
                .ok_or(DecodeError::InvalidU32)?;
            if b & 0b1000_0000 == 0 {
                break;
            }
            bits += 7;
            if bits >= 32 {
                return Err(DecodeError::InvalidU32);
            }
        }
        Ok(n)
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if n > self.len() {
            return Err(DecodeError::EndOfBytes);
        }

        let start = self.position;
        self.position += n;
        Ok(&self.data[start..][..n])
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        let n = buf.len();
        if n > self.len() {
            return Err(DecodeError::EndOfBytes);
        }

        buf.copy_from_slice(&self.data[self.position..][..n]);
        self.position += n;
        Ok(())
    }

    pub fn validate_preamble(&mut self) -> Result<(), DecodeError> {
        let magic = self.read_bytes(4)?;
        if magic != b"\0asm" {
            return Err(DecodeError::InvalidMagicNumber);
        }

        let version = self.read_bytes(4)?;
        if version != b"\x01\0\0\0" {
            return Err(DecodeError::UnsupportedVersion);
        }

        Ok(())
    }

    pub fn read_section_reader(&mut self) -> Result<(SectionId, Self), DecodeError> {
        let section_id = SectionId::new(self.read_u8()?)?;
        let size = self.read_u32()? as usize;
        if size > self.len() {
            return Err(DecodeError::TooLargeSectionSize { section_id, size });
        }

        let reader = Self::new(&self.data[self.position..][..size]);
        self.position += size;

        Ok((section_id, reader))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionId {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
}

impl SectionId {
    pub fn new(value: u8) -> Result<Self, DecodeError> {
        match value {
            0 => Ok(Self::Custom),
            1 => Ok(Self::Type),
            2 => Ok(Self::Import),
            3 => Ok(Self::Function),
            4 => Ok(Self::Table),
            5 => Ok(Self::Memory),
            6 => Ok(Self::Global),
            7 => Ok(Self::Export),
            8 => Ok(Self::Start),
            9 => Ok(Self::Element),
            10 => Ok(Self::Code),
            11 => Ok(Self::Data),
            _ => Err(DecodeError::InvalidSectionId { value }),
        }
    }
}

#[derive(Debug)]
pub struct Module<A> {
    allocator: A,
}

impl<A: Allocator> Module<A> {
    pub fn decode(wasm: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = ByteReader::new(wasm);
        reader.validate_preamble()?;

        while !reader.is_empty() {
            let (_id, _section_reader) = reader.read_section_reader()?;
        }

        todo!()
    }
}
