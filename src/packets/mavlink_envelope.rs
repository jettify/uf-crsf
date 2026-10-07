use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;
use heapless::Vec;

/// Represents a CRSF `MAVLink` Envelope packet (type 0xAA).
///
/// This packet is used to transfer `MAVLink` protocol frames over CRSF.
/// Since `MAVLink` frames can be larger than a single CRSF frame, they are
/// broken up into chunks.
/// Uses a short header: chunk info and data size immediately follow the type,
/// without destination or origin bytes.
/// Chunk indexing and reassembly are managed by the caller.
#[derive(Clone, Debug, PartialEq)]
pub struct MavlinkEnvelope {
    /// Zero-based index of the last chunk (0..=15).
    /// A single-chunk frame uses `total_chunks = 0` and `current_chunk = 0`.
    pub total_chunks: u8,
    /// Zero-based index of the current chunk; must not exceed `total_chunks`.
    pub current_chunk: u8,
    /// The MAVLink data payload for this chunk.
    data: Vec<u8, 58>,
}

impl MavlinkEnvelope {
    /// Creates a new MavlinkEnvelope packet from a slice of data.
    ///
    /// The data slice must not be longer than 58 bytes.
    /// Both chunk indexes must fit four bits, with `current_chunk <= total_chunks`.
    pub fn new(total_chunks: u8, current_chunk: u8, data: &[u8]) -> Result<Self, CrsfParsingError> {
        Self::validate_chunks(total_chunks, current_chunk)?;
        if data.len() > 58 {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let mut d = Vec::new();
        d.extend_from_slice(data)
            .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self {
            total_chunks,
            current_chunk,
            data: d,
        })
    }

    /// Returns the MAVLink data as a slice.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn validate_chunks(total_chunks: u8, current_chunk: u8) -> Result<(), CrsfParsingError> {
        if total_chunks > 15 || current_chunk > 15 || current_chunk > total_chunks {
            return Err(CrsfParsingError::InvalidPayload);
        }
        Ok(())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for MavlinkEnvelope {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "MavlinkEnvelope {{ total_chunks: {}, current_chunk: {} data: {} }}",
            self.total_chunks,
            self.current_chunk,
            self.data(),
        )
    }
}

impl CrsfPacket for MavlinkEnvelope {
    const PACKET_TYPE: PacketType = PacketType::MavlinkEnvelope;
    // The payload must contain at least the chunk info and data size bytes.
    const MIN_PAYLOAD_SIZE: usize = 2;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        // Public chunk fields may have been changed after construction.
        Self::validate_chunks(self.total_chunks, self.current_chunk)?;
        let data_size = self.data.len();
        if buffer.len() < 2 + data_size {
            return Err(CrsfParsingError::BufferOverflow);
        }

        // Pack total_chunks and current_chunk into a single byte
        buffer[0] = (self.total_chunks << 4) | self.current_chunk;
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

        Self::new(total_chunks, current_chunk, &data[2..2 + data_size])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_wire_chunk_indexes() {
        for chunk_info in 0..=u8::MAX {
            let total = chunk_info >> 4;
            let current = chunk_info & 0x0f;
            let decoded = MavlinkEnvelope::from_bytes(&[chunk_info, 0]);
            let constructed = MavlinkEnvelope::new(total, current, &[]);
            if current > total {
                assert_eq!(decoded, Err(CrsfParsingError::InvalidPayload));
                assert_eq!(constructed, Err(CrsfParsingError::InvalidPayload));
            } else {
                assert_eq!(decoded, constructed);
                let mut buffer = [0xaa; 2];
                assert_eq!(constructed.unwrap().to_bytes(&mut buffer), Ok(2));
                assert_eq!(buffer, [chunk_info, 0]);
            }
        }
    }

    #[test]
    fn test_invalid_public_chunk_fields() {
        for (total, current) in [(16, 0), (0, 16), (16, 16), (255, 255), (0, 1), (2, 3)] {
            assert_eq!(
                MavlinkEnvelope::new(total, current, &[]),
                Err(CrsfParsingError::InvalidPayload)
            );
            let mut packet = MavlinkEnvelope::new(0, 0, &[1, 2]).unwrap();
            packet.total_chunks = total;
            packet.current_chunk = current;
            let mut buffer = [0xaa; 4];
            assert_eq!(
                packet.to_bytes(&mut buffer),
                Err(CrsfParsingError::InvalidPayload)
            );
            assert_eq!(buffer, [0xaa; 4]);
        }
    }

    #[test]
    fn test_data_length_and_buffer_boundaries() {
        let mut oversized = [0; 61];
        oversized[1] = 59;
        assert_eq!(
            MavlinkEnvelope::from_bytes(&oversized),
            Err(CrsfParsingError::InvalidPayloadLength)
        );
        let packet = MavlinkEnvelope::new(0, 0, &[1, 2]).unwrap();
        assert_eq!(
            packet.to_bytes(&mut [0; 3]),
            Err(CrsfParsingError::BufferOverflow)
        );
        let decoded = MavlinkEnvelope::from_bytes(&[0, 2, 1, 2, 0xaa]).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn test_mavlink_envelope_to_bytes() {
        let data = [1, 2, 3, 4];
        let packet = MavlinkEnvelope::new(5, 2, &data).unwrap();

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

        let expected_data = [1, 2, 3, 4];
        assert_eq!(packet.total_chunks, 5);
        assert_eq!(packet.current_chunk, 2);
        assert_eq!(packet.data(), &expected_data);
    }

    #[test]
    fn test_mavlink_envelope_round_trip() {
        let data = [0xFE, 0xED, 0xBE, 0xEF];
        let packet = MavlinkEnvelope::new(10, 9, &data).unwrap();

        let mut buffer = [0u8; 60];
        let len = packet.to_bytes(&mut buffer).unwrap();
        let round_trip_packet = MavlinkEnvelope::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(packet, round_trip_packet);
    }

    #[test]
    fn test_empty_data() {
        let packet = MavlinkEnvelope::new(1, 0, &[]).unwrap();

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
        let payload = [0xAB; 58];
        let packet = MavlinkEnvelope::new(15, 15, &payload).unwrap();

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

    #[test]
    fn test_mavlink_envelope_new_data_too_long() {
        let payload = [0xAB; 59];
        let result = MavlinkEnvelope::new(1, 0, &payload);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }
}
