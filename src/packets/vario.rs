use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a Variometer Sensor packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VariometerSensor {
    /// Vertical speed in cm/s.
    pub v_speed: i16,
}

impl VariometerSensor {
    pub fn new(v_speed: i16) -> Result<Self, CrsfParsingError> {
        Ok(Self { v_speed })
    }
}

impl CrsfPacket for VariometerSensor {
    const PACKET_TYPE: PacketType = PacketType::Vario;
    const MIN_PAYLOAD_SIZE: usize = 2;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[..2].copy_from_slice(&self.v_speed.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            v_speed: i16::from_be_bytes([data[0], data[1]]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vario_from_bytes() {
        assert_eq!(VariometerSensor::MIN_PAYLOAD_SIZE, 2);
        let data: [u8; 2] = [0x01, 0x02];
        let packet = VariometerSensor::from_bytes(&data).unwrap();
        assert_eq!(packet.v_speed, 0x0102);
    }

    #[test]
    fn test_vario_to_bytes() {
        let packet = VariometerSensor { v_speed: -1000 };
        let mut buffer = [0u8; 2];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 2);
        let expected: [u8; 2] = [0xFC, 0x18];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_vario_round_trip() {
        let packet = VariometerSensor { v_speed: 1234 };
        let mut buffer = [0u8; 2];
        packet.to_bytes(&mut buffer).unwrap();
        let round_trip = VariometerSensor::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip);
    }

    #[test]
    fn test_vario_from_bytes_too_small() {
        let data: [u8; 1] = [0; 1];
        let result = VariometerSensor::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }

    #[test]
    fn test_vario_to_bytes_too_small() {
        let packet = VariometerSensor { v_speed: 1 };
        let mut buffer = [0u8; 1];
        let result = packet.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }
}
