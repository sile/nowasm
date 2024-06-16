use crate::DecodeError;

#[derive(Debug)]
pub struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data, position: 0 }
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.position
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        let v = self
            .data
            .get(self.position)
            .copied()
            .ok_or(DecodeError::EndOfBytes)?;
        self.position += 1;
        Ok(v)
    }

    pub fn read(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        let v = self
            .data
            .get(self.position..self.position + n)
            .ok_or(DecodeError::EndOfBytes)?;
        self.position += n;
        Ok(v)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        buf.copy_from_slice(self.read(buf.len())?);
        Ok(())
    }

    pub fn read_u64(&mut self) -> Result<u64, DecodeError> {
        self.read_integer_u(64)
    }

    pub fn read_i64(&mut self) -> Result<i64, DecodeError> {
        self.read_integer_s(64)
    }

    pub fn read_u32(&mut self) -> Result<u32, DecodeError> {
        self.read_integer_u(32).map(|n| n as u32)
    }

    pub fn read_i32(&mut self) -> Result<i32, DecodeError> {
        self.read_integer_s(32).map(|n| n as i32)
    }

    pub fn read_f32(&mut self) -> Result<f32, DecodeError> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    pub fn read_f64(&mut self) -> Result<f64, DecodeError> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    pub fn read_usize(&mut self) -> Result<usize, DecodeError> {
        self.read_u32().map(|v| v as usize)
    }

    pub fn read_integer_u(&mut self, bits: usize) -> Result<u64, DecodeError> {
        let mut n = 0u64;
        let mut offset = 0;
        loop {
            let b = self.read_u8()?;
            let v = (b as u64 & 0b0111_1111) << offset;

            if b & 0b1000_0000 == 0 {
                let remaining_bits = bits - offset;
                if b >= 1u8.checked_shl(remaining_bits as u32).unwrap_or(u8::MAX) {
                    return Err(DecodeError::MalformedInteger);
                }
                n += v;
                break;
            }

            n += v;
            offset += 7;
            if offset >= bits {
                return Err(DecodeError::MalformedInteger);
            }
        }
        Ok(n)
    }

    pub fn read_integer_s(&mut self, bits: usize) -> Result<i64, DecodeError> {
        let mut n = 0i64;
        let mut offset = 0;
        loop {
            let b = self.read_u8()?;

            if b < 0b0100_0000 {
                let remaining_bits = bits - offset;
                if b >= 1u8
                    .checked_shl(remaining_bits as u32 - 1)
                    .unwrap_or(u8::MAX)
                {
                    return Err(DecodeError::MalformedInteger);
                }
                n += (b as i64) << offset;
                break;
            } else if b < 0b1000_0000 {
                let remaining_bits = bits - offset;
                if remaining_bits <= 8 && b < (0b1000_0000 - (1u8 << (remaining_bits as u32 - 1))) {
                    return Err(DecodeError::MalformedInteger);
                }
                n += ((b as i64) - 0b1000_000) << offset;
                break;
            }

            n += (b as i64 & 0b0111_1111) << offset;
            offset += 7;
            if offset >= bits {
                return Err(DecodeError::MalformedInteger);
            }
        }
        Ok(n)
    }
}
