use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;

/// Represents a Barometer packet (frame type `0x11`).
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Barometer {
    /// Barometric pressure in pascals.
    pub pressure_pa: i32,
    /// Barometer temperature in centidegrees Celsius.
    pub baro_temp: i32,
}

impl Barometer {
    pub fn new(pressure_pa: i32, baro_temp: i32) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            pressure_pa,
            baro_temp,
        })
    }
}

impl CrsfPacket for Barometer {
    const PACKET_TYPE: PacketType = PacketType::Barometer;
    const MIN_PAYLOAD_SIZE: usize = 8;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            pressure_pa: i32::from_be_bytes(data[0..4].try_into().expect("infallible")),
            baro_temp: i32::from_be_bytes(data[4..8].try_into().expect("infallible")),
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0..4].copy_from_slice(&self.pressure_pa.to_be_bytes());
        buffer[4..8].copy_from_slice(&self.baro_temp.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barometer_new() {
        let packet = Barometer::new(101_325, 2_500).unwrap();
        assert_eq!(packet.pressure_pa, 101_325);
        assert_eq!(packet.baro_temp, 2_500);
    }

    #[test]
    fn test_barometer_to_bytes() {
        let packet = Barometer {
            pressure_pa: 101_325,
            baro_temp: -500,
        };
        let mut buffer = [0u8; Barometer::MIN_PAYLOAD_SIZE];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, Barometer::MIN_PAYLOAD_SIZE);
        assert_eq!(buffer, [0x00, 0x01, 0x8B, 0xCD, 0xFF, 0xFF, 0xFE, 0x0C]);
    }

    #[test]
    fn test_barometer_from_bytes() {
        let data = [0x00, 0x01, 0x8B, 0xCD, 0x00, 0x00, 0x09, 0xC4];
        let packet = Barometer::from_bytes(&data).unwrap();
        assert_eq!(packet.pressure_pa, 101_325);
        assert_eq!(packet.baro_temp, 2_500);
    }

    #[test]
    fn test_barometer_round_trip() {
        let packet = Barometer {
            pressure_pa: -10_000,
            baro_temp: 3_333,
        };
        let mut buffer = [0u8; Barometer::MIN_PAYLOAD_SIZE];
        packet.to_bytes(&mut buffer).unwrap();
        let round_trip = Barometer::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip);
    }

    #[test]
    fn test_barometer_to_bytes_too_small() {
        let packet = Barometer {
            pressure_pa: 1,
            baro_temp: 2,
        };
        let mut buffer = [0u8; 7];
        let result = packet.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }

    #[test]
    fn test_barometer_from_bytes_invalid_length() {
        let data = [0u8; 7];
        let result = Barometer::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }
}
