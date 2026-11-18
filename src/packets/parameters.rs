use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;
use heapless::Vec;

/// Requests a parameter definition chunk (frame type 0x2C).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ParameterRead {
    pub dst_addr: u8,
    pub src_addr: u8,
    /// Parameter zero denotes the root folder on devices that support it.
    pub parameter_number: u8,
    /// Zero-based chunk index; request zero first.
    pub chunk_number: u8,
}

impl ParameterRead {
    pub fn new(
        dst_addr: u8,
        src_addr: u8,
        parameter_number: u8,
        chunk_number: u8,
    ) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            dst_addr,
            src_addr,
            parameter_number,
            chunk_number,
        })
    }
}

impl CrsfPacket for ParameterRead {
    const PACKET_TYPE: PacketType = PacketType::ParameterRead;
    const MIN_PAYLOAD_SIZE: usize = 4;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Self::new(data[0], data[1], data[2], data[3])
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[..4].copy_from_slice(&[
            self.dst_addr,
            self.src_addr,
            self.parameter_number,
            self.chunk_number,
        ]);
        Ok(4)
    }
}

/// One raw parameter definition chunk (frame type 0x2B).
///
/// Reassembly and parameter type decoding are managed by the caller. A chunk
/// may split any part of a definition, including its name or type fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterSettingsEntry {
    pub dst_addr: u8,
    pub src_addr: u8,
    pub parameter_number: u8,
    /// Number of chunks still to read after this one; zero means the last chunk.
    pub chunks_remaining: u8,
    data: Vec<u8, 56>,
}

impl ParameterSettingsEntry {
    /// Creates a raw chunk with at most 56 data bytes, keeping the frame <=64 bytes.
    pub fn new(
        dst_addr: u8,
        src_addr: u8,
        parameter_number: u8,
        chunks_remaining: u8,
        data: &[u8],
    ) -> Result<Self, CrsfParsingError> {
        let data = Vec::from_slice(data).map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self {
            dst_addr,
            src_addr,
            parameter_number,
            chunks_remaining,
            data,
        })
    }

    /// Raw definition bytes for this chunk, preserved without interpretation.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl CrsfPacket for ParameterSettingsEntry {
    const PACKET_TYPE: PacketType = PacketType::ParameterSettingsEntry;
    const MIN_PAYLOAD_SIZE: usize = 4;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Self::new(data[0], data[1], data[2], data[3], &data[4..])
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let len = Self::MIN_PAYLOAD_SIZE + self.data.len();
        if buffer.len() < len {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[..4].copy_from_slice(&[
            self.dst_addr,
            self.src_addr,
            self.parameter_number,
            self.chunks_remaining,
        ]);
        buffer[4..len].copy_from_slice(self.data());
        Ok(len)
    }
}

/// A raw parameter value write or value acknowledgement (frame type 0x2D).
///
/// The caller encodes/decodes the value using the parameter's definition.
/// Command writes receive a ParameterSettingsEntry response instead of a value
/// acknowledgement. This codec does not execute writes or generate responses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterWrite {
    pub dst_addr: u8,
    pub src_addr: u8,
    pub parameter_number: u8,
    data: Vec<u8, 57>,
}

impl ParameterWrite {
    /// Creates a raw value packet with at most 57 data bytes.
    /// Value size and validity depend on the parameter type and are caller-checked.
    pub fn new(
        dst_addr: u8,
        src_addr: u8,
        parameter_number: u8,
        data: &[u8],
    ) -> Result<Self, CrsfParsingError> {
        let data = Vec::from_slice(data).map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self {
            dst_addr,
            src_addr,
            parameter_number,
            data,
        })
    }

    /// Raw value bytes, including any string terminator supplied by the caller.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl CrsfPacket for ParameterWrite {
    const PACKET_TYPE: PacketType = PacketType::ParameterWrite;
    const MIN_PAYLOAD_SIZE: usize = 3;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Self::new(data[0], data[1], data[2], &data[3..])
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let len = Self::MIN_PAYLOAD_SIZE + self.data.len();
        if buffer.len() < len {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[..3].copy_from_slice(&[self.dst_addr, self.src_addr, self.parameter_number]);
        buffer[3..len].copy_from_slice(self.data());
        Ok(len)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ParameterSettingsEntry {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ParameterSettingsEntry {{ dst_addr: {}, src_addr: {}, parameter_number: {}, chunks_remaining: {}, data: {} }}",
            self.dst_addr, self.src_addr, self.parameter_number, self.chunks_remaining, self.data(),
        )
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ParameterWrite {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ParameterWrite {{ dst_addr: {}, src_addr: {}, parameter_number: {}, data: {} }}",
            self.dst_addr,
            self.src_addr,
            self.parameter_number,
            self.data(),
        )
    }
}
