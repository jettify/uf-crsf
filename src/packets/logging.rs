use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;
use core::mem::size_of;
use heapless::Vec;

/// Represents a Logging packet (frame type 0x34).
#[derive(Clone, Debug, PartialEq)]
pub struct Logging {
    pub dst_addr: u8,
    pub src_addr: u8,
    pub logtype: u16,
    pub timestamp: u32,
    params: Vec<u32, 13>,
}

impl Logging {
    /// Creates a new Logging packet.
    pub fn new(
        dst_addr: u8,
        src_addr: u8,
        logtype: u16,
        timestamp: u32,
        params: &[u32],
    ) -> Result<Self, CrsfParsingError> {
        if params.len() > 13 {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let mut p = Vec::new();
        p.extend_from_slice(params)
            .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self {
            dst_addr,
            src_addr,
            logtype,
            timestamp,
            params: p,
        })
    }

    /// Returns the logging parameters as a slice.
    pub fn params(&self) -> &[u32] {
        &self.params
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Logging {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Logging {{ dst_addr: {}, src_addr: {}, logtype: {}, timestamp: {}, params: {} }}",
            self.dst_addr,
            self.src_addr,
            self.logtype,
            self.timestamp,
            self.params(),
        )
    }
}

impl CrsfPacket for Logging {
    const PACKET_TYPE: PacketType = PacketType::Logging;
    const MIN_PAYLOAD_SIZE: usize = size_of::<u8>() * 2 + size_of::<u16>() + size_of::<u32>();

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        if !(data.len() - Self::MIN_PAYLOAD_SIZE).is_multiple_of(size_of::<u32>()) {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let dst_addr = data[0];
        let src_addr = data[1];
        let logtype = u16::from_be_bytes(data[2..4].try_into().expect("infallible"));
        let timestamp = u32::from_be_bytes(data[4..8].try_into().expect("infallible"));

        let mut params = Vec::new();
        for chunk in data[8..].chunks_exact(4) {
            let param = u32::from_be_bytes(chunk.try_into().expect("infallible"));
            if params.push(param).is_err() {
                // This would mean we have more params than our Vec can hold.
                return Err(CrsfParsingError::InvalidPayloadLength);
            }
        }

        Ok(Self {
            dst_addr,
            src_addr,
            logtype,
            timestamp,
            params,
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let params_len = core::mem::size_of_val(self.params());
        let payload_len = Self::MIN_PAYLOAD_SIZE + params_len;
        if buffer.len() < payload_len {
            return Err(CrsfParsingError::BufferOverflow);
        }

        buffer[0] = self.dst_addr;
        buffer[1] = self.src_addr;
        buffer[2..4].copy_from_slice(&self.logtype.to_be_bytes());
        buffer[4..8].copy_from_slice(&self.timestamp.to_be_bytes());

        for (i, param) in self.params().iter().enumerate() {
            let offset = Self::MIN_PAYLOAD_SIZE + i * size_of::<u32>();
            buffer[offset..offset + size_of::<u32>()].copy_from_slice(&param.to_be_bytes());
        }

        Ok(payload_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_from_bytes_no_params() {
        assert_eq!(Logging::MIN_PAYLOAD_SIZE, 8);
        let data: [u8; 8] = [
            0xEA, 0xEE, // dst_addr, src_addr
            0x01, 0x02, // logtype
            0x03, 0x04, 0x05, 0x06, // timestamp
        ];
        let packet = Logging::from_bytes(&data).unwrap();
        assert_eq!(packet.dst_addr, 0xEA);
        assert_eq!(packet.src_addr, 0xEE);
        assert_eq!(packet.logtype, 0x0102);
        assert_eq!(packet.timestamp, 0x03040506);
        assert!(packet.params().is_empty());
    }

    #[test]
    fn test_logging_from_bytes_with_params() {
        let data: [u8; 16] = [
            0xEA, 0xEE, // dst_addr, src_addr
            0x01, 0x02, // logtype
            0x03, 0x04, 0x05, 0x06, // timestamp
            0x07, 0x08, 0x09, 0x0A, // param1
            0x0B, 0x0C, 0x0D, 0x0E, // param2
        ];
        let packet = Logging::from_bytes(&data).unwrap();
        assert_eq!(packet.dst_addr, 0xEA);
        assert_eq!(packet.src_addr, 0xEE);
        assert_eq!(packet.logtype, 0x0102);
        assert_eq!(packet.timestamp, 0x03040506);
        assert_eq!(packet.params(), &[0x0708090A, 0x0B0C0D0E]);
    }

    #[test]
    fn test_logging_to_bytes_no_params() {
        let packet = Logging::new(0xEA, 0xEE, 0x1234, 0x56789ABC, &[]).unwrap();
        let mut buffer = [0u8; 8];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 8);
        let expected: [u8; 8] = [0xEA, 0xEE, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_logging_to_bytes_with_params() {
        let params = [0x11223344, 0x55667788];
        let packet = Logging::new(0xEA, 0xEE, 0xABCD, 0xDEADBEEF, &params).unwrap();
        let mut buffer = [0u8; 16];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 16);
        let expected: [u8; 16] = [
            0xEA, 0xEE, // dst_addr, src_addr
            0xAB, 0xCD, // logtype
            0xDE, 0xAD, 0xBE, 0xEF, // timestamp
            0x11, 0x22, 0x33, 0x44, // param1
            0x55, 0x66, 0x77, 0x88, // param2
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_logging_round_trip() {
        let params = [1, 2, 3];
        let packet = Logging::new(0xEA, 0xEE, 123, 456, &params).unwrap();
        let mut buffer = [0u8; 20];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 20);
        let round_trip = Logging::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(packet, round_trip);
    }

    #[test]
    fn test_invalid_payload_length_too_short() {
        let data = [0u8; 7];
        assert_eq!(
            Logging::from_bytes(&data),
            Err(CrsfParsingError::InvalidPayloadLength)
        );
    }

    #[test]
    fn test_invalid_payload_length_not_multiple_of_4() {
        let data = [0u8; 9];
        assert_eq!(
            Logging::from_bytes(&data),
            Err(CrsfParsingError::InvalidPayloadLength)
        );
    }
}
