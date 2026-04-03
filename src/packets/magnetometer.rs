use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;

/// Represents a Magnetometer packet (frame type `0x12`).
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Magnetometer {
    /// Magnetic field X axis in milligauss * 3.
    pub field_x: i16,
    /// Magnetic field Y axis in milligauss * 3.
    pub field_y: i16,
    /// Magnetic field Z axis in milligauss * 3.
    pub field_z: i16,
}

impl Magnetometer {
    pub fn new(field_x: i16, field_y: i16, field_z: i16) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            field_x,
            field_y,
            field_z,
        })
    }
}

impl CrsfPacket for Magnetometer {
    const PACKET_TYPE: PacketType = PacketType::Magnetometer;
    const MIN_PAYLOAD_SIZE: usize = 6;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            field_x: i16::from_be_bytes(data[0..2].try_into().expect("infallible")),
            field_y: i16::from_be_bytes(data[2..4].try_into().expect("infallible")),
            field_z: i16::from_be_bytes(data[4..6].try_into().expect("infallible")),
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0..2].copy_from_slice(&self.field_x.to_be_bytes());
        buffer[2..4].copy_from_slice(&self.field_y.to_be_bytes());
        buffer[4..6].copy_from_slice(&self.field_z.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnetometer_new() {
        let packet = Magnetometer::new(1_000, -500, 123).unwrap();
        assert_eq!(packet.field_x, 1_000);
        assert_eq!(packet.field_y, -500);
        assert_eq!(packet.field_z, 123);
    }

    #[test]
    fn test_magnetometer_to_bytes() {
        let packet = Magnetometer {
            field_x: 1_000,
            field_y: -500,
            field_z: 123,
        };
        let mut buffer = [0u8; Magnetometer::MIN_PAYLOAD_SIZE];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, Magnetometer::MIN_PAYLOAD_SIZE);
        assert_eq!(buffer, [0x03, 0xE8, 0xFE, 0x0C, 0x00, 0x7B]);
    }

    #[test]
    fn test_magnetometer_from_bytes() {
        let data = [0x03, 0xE8, 0xFE, 0x0C, 0x00, 0x7B];
        let packet = Magnetometer::from_bytes(&data).unwrap();
        assert_eq!(packet.field_x, 1_000);
        assert_eq!(packet.field_y, -500);
        assert_eq!(packet.field_z, 123);
    }

    #[test]
    fn test_magnetometer_round_trip() {
        let packet = Magnetometer {
            field_x: i16::MIN,
            field_y: 0,
            field_z: i16::MAX,
        };
        let mut buffer = [0u8; Magnetometer::MIN_PAYLOAD_SIZE];
        packet.to_bytes(&mut buffer).unwrap();
        let round_trip = Magnetometer::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip);
    }

    #[test]
    fn test_magnetometer_to_bytes_too_small() {
        let packet = Magnetometer {
            field_x: 1,
            field_y: 2,
            field_z: 3,
        };
        let mut buffer = [0u8; 5];
        let result = packet.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }

    #[test]
    fn test_magnetometer_from_bytes_invalid_length() {
        let data = [0u8; 5];
        let result = Magnetometer::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }
}
