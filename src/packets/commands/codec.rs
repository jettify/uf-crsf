use super::screen::CommandString;
use crate::CrsfParsingError;

// Small checked cursors keep variable-length strings and packed fields bounded
// without allocating or reading into either CRC byte.
pub(super) struct Reader<'a> {
    pub(super) remaining: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(super) fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    pub(super) fn take(&mut self, count: usize) -> Result<&'a [u8], CrsfParsingError> {
        if self.remaining.len() < count {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let (value, remaining) = self.remaining.split_at(count);
        self.remaining = remaining;
        Ok(value)
    }

    pub(super) fn byte(&mut self) -> Result<u8, CrsfParsingError> {
        Ok(self.take(1)?[0])
    }

    pub(super) fn boolean(&mut self) -> Result<bool, CrsfParsingError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }

    pub(super) fn u16(&mut self) -> Result<u16, CrsfParsingError> {
        let bytes = self.take(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    pub(super) fn u32(&mut self) -> Result<u32, CrsfParsingError> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(super) fn string(&mut self) -> Result<CommandString, CrsfParsingError> {
        let len = self
            .remaining
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(CrsfParsingError::InvalidPayloadLength)?;
        let bytes = self.take(len + 1)?;
        CommandString::new(&bytes[..len])
    }
}

pub(super) struct Writer<'a> {
    buffer: &'a mut [u8],
    pub(super) position: usize,
}

impl<'a> Writer<'a> {
    pub(super) fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub(super) fn bytes(&mut self, bytes: &[u8]) -> Result<(), CrsfParsingError> {
        let end = self.position + bytes.len();
        let target = self
            .buffer
            .get_mut(self.position..end)
            .ok_or(CrsfParsingError::BufferOverflow)?;
        target.copy_from_slice(bytes);
        self.position = end;
        Ok(())
    }

    pub(super) fn byte(&mut self, byte: u8) -> Result<(), CrsfParsingError> {
        self.bytes(&[byte])
    }

    pub(super) fn string(&mut self, string: &CommandString) -> Result<(), CrsfParsingError> {
        self.bytes(string.as_bytes())?;
        self.byte(0)
    }
}
