use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;
use heapless::Vec;

/// Represents a CRSF `MAVLink` Envelope packet (type 0xAA).
///
/// This packet is used to transfer `MAVLink` protocol frames over CRSF.
/// Since `MAVLink` frames can be larger than a single CRSF frame, they are
/// broken up into chunks.
#[derive(Clone, Debug, PartialEq)]
pub struct MavlinkEnvelope {
    /// Total number of chunks for the `MAVLink` frame.
    pub total_chunks: u8,
    /// The index of the current chunk (0-based).
    pub current_chunk: u8,
    /// The `MAVLink` data payload for this chunk.
    pub data: Vec<u8, 58>,
}

#[cfg(feature = "defmt")]
impl defmt::Format for MavlinkEnvelope {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "MavlinkEnvelope {{ total_chunks: {}, current_chunk: {} data: {} }}",
            self.total_chunks,
            self.current_chunk,
            self.data.as_slice(),
        )
    }
}

impl CrsfPacket for MavlinkEnvelope {
    const PACKET_TYPE: PacketType = PacketType::MavlinkEnvelope;
    // The payload must contain at least the chunk info and data size bytes.
    const MIN_PAYLOAD_SIZE: usize = 2;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let data_size = self.data.len();
        if buffer.len() < 2 + data_size {
            return Err(CrsfParsingError::BufferOverflow);
        }

        // Pack total_chunks and current_chunk into a single byte
        buffer[0] = (self.total_chunks << 4) | (self.current_chunk & 0x0F);
        buffer[1] = data_size as u8;
        buffer[2..2 + data_size].copy_from_slice(&self.data);

        Ok(2 + data_size)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let total_chunks = data[0] >> 4;
        let current_chunk = data[0] & 0x0F;
        let data_size = data[1] as usize;

        if data.len() < 2 + data_size {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let mut payload_data = Vec::new();
        payload_data
            .extend_from_slice(&data[2..2 + data_size])
            .map_err(|()| CrsfParsingError::InvalidPayloadLength)?;

        Ok(Self {
            total_chunks,
            current_chunk,
            data: payload_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mavlink_envelope_to_bytes() {
        let mut data: Vec<u8, 58> = Vec::new();
        data.extend_from_slice(&[1, 2, 3, 4]).unwrap();
        let packet = MavlinkEnvelope {
            total_chunks: 5,
            current_chunk: 2,
            data,
        };

        let mut buffer = [0u8; 6];
        let len = packet.to_bytes(&mut buffer).unwrap();

        assert_eq!(len, 6);
        // total_chunks: 5 (0b0101), current_chunk: 2 (0b0010) -> 0b01010010 = 0x52
        // data_size: 4
        assert_eq!(buffer, [0x52, 4, 1, 2, 3, 4]);
    }

    #[test]
    fn test_mavlink_envelope_from_bytes() {
        let data: [u8; 6] = [0x52, 4, 1, 2, 3, 4];
        let packet = MavlinkEnvelope::from_bytes(&data).unwrap();

        let mut expected_data: Vec<u8, 58> = Vec::new();
        expected_data.extend_from_slice(&[1, 2, 3, 4]).unwrap();
        assert_eq!(packet.total_chunks, 5);
        assert_eq!(packet.current_chunk, 2);
        assert_eq!(packet.data, expected_data);
    }

    #[test]
    fn test_mavlink_envelope_round_trip() {
        let mut data: Vec<u8, 58> = Vec::new();
        data.extend_from_slice(&[0xFE, 0xED, 0xBE, 0xEF]).unwrap();
        let packet = MavlinkEnvelope {
            total_chunks: 10,
            current_chunk: 9,
            data,
        };

        let mut buffer = [0u8; 60];
        let len = packet.to_bytes(&mut buffer).unwrap();
        let round_trip_packet = MavlinkEnvelope::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(packet, round_trip_packet);
    }

    #[test]
    fn test_empty_data() {
        let packet = MavlinkEnvelope {
            total_chunks: 1,
            current_chunk: 0,
            data: Vec::new(),
        };

        let mut buffer = [0u8; 2];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 2);
        // total_chunks: 1 (0b0001), current_chunk: 0 (0b0000) -> 0b00010000 = 0x10
        assert_eq!(buffer, [0x10, 0]);

        let round_trip_packet = MavlinkEnvelope::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip_packet);
    }

    #[test]
    fn test_max_data() {
        let mut data: Vec<u8, 58> = Vec::new();
        let payload = [0xAB; 58];
        data.extend_from_slice(&payload).unwrap();

        let packet = MavlinkEnvelope {
            total_chunks: 15,
            current_chunk: 15,
            data,
        };

        let mut buffer = [0u8; 60];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 60);

        // total_chunks: 15 (0b1111), current_chunk: 15 (0b1111) -> 0b11111111 = 0xFF
        assert_eq!(buffer[0], 0xFF);
        assert_eq!(buffer[1], 58);
        assert_eq!(&buffer[2..], payload);

        let round_trip_packet = MavlinkEnvelope::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip_packet);
    }

    #[test]
    fn test_from_bytes_invalid_len() {
        let data: [u8; 1] = [0x10];
        let result = MavlinkEnvelope::from_bytes(&data);
        assert!(matches!(
            result,
            Err(CrsfParsingError::InvalidPayloadLength)
        ));

        let data: [u8; 5] = [0x10, 4, 1, 2, 3]; // data_size is 4, but only 3 bytes provided
        let result = MavlinkEnvelope::from_bytes(&data);
        assert!(matches!(
            result,
            Err(CrsfParsingError::InvalidPayloadLength)
        ));
    }
}
