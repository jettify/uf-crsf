use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;
use core::mem::size_of;

/// Represents an Attitude packet (frame type 0x1E).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Attitude {
    /// Pitch angle in 100 µrad units.
    pub pitch: i16,
    /// Roll angle in 100 µrad units.
    pub roll: i16,
    /// Yaw angle in 100 µrad units.
    pub yaw: i16,
}

impl CrsfPacket for Attitude {
    const PACKET_TYPE: PacketType = PacketType::Attitude;
    const MIN_PAYLOAD_SIZE: usize = size_of::<i16>() * 3;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Ok(Self {
            pitch: i16::from_be_bytes(data[0..2].try_into().expect("infallible")),
            roll: i16::from_be_bytes(data[2..4].try_into().expect("infallible")),
            yaw: i16::from_be_bytes(data[4..6].try_into().expect("infallible")),
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        if buffer.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[0..2].copy_from_slice(&self.pitch.to_be_bytes());
        buffer[2..4].copy_from_slice(&self.roll.to_be_bytes());
        buffer[4..6].copy_from_slice(&self.yaw.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attitude_from_bytes() {
        let data: [u8; 6] = [
            0x01, 0x02, // pitch
            0x03, 0x04, // roll
            0x05, 0x06, // yaw
        ];
        let packet = Attitude::from_bytes(&data).unwrap();
        assert_eq!(packet.pitch, 0x0102);
        assert_eq!(packet.roll, 0x0304);
        assert_eq!(packet.yaw, 0x0506);
    }

    #[test]
    fn test_attitude_to_bytes() {
        let packet = Attitude {
            pitch: -1000,
            roll: 1000,
            yaw: 31415,
        };
        let mut buffer = [0u8; 6];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 6);
        let expected: [u8; 6] = [
            0xFC, 0x18, // -1000
            0x03, 0xE8, // 1000
            0x7A, 0xB7, // 31415
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_attitude_round_trip() {
        let packet = Attitude {
            pitch: 123,
            roll: -456,
            yaw: 789,
        };
        let mut buffer = [0u8; 6];
        packet.to_bytes(&mut buffer).unwrap();
        let round_trip = Attitude::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip);
    }
}
