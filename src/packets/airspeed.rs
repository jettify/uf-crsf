use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents an Airspeed packet.
///
/// This packet is used to transmit airspeed data from the vehicle.
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AirSpeed {
    /// Airspeed in 0.1 * km/h (hectometers/h).
    pub speed: u16,
}

impl AirSpeed {
    pub fn new(speed: u16) -> Self {
        Self { speed }
    }
}

impl CrsfPacket for AirSpeed {
    const PACKET_TYPE: PacketType = PacketType::AirSpeed;
    const MIN_PAYLOAD_SIZE: usize = size_of::<u16>();

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0..2].copy_from_slice(&self.speed.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        let speed = data
            .get(0..2)
            .and_then(|slice| slice.try_into().ok())
            .map(u16::from_be_bytes)
            .ok_or(CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self { speed })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airspeed_to_bytes() {
        assert_eq!(AirSpeed::MIN_PAYLOAD_SIZE, 2);
        let airspeed = AirSpeed { speed: 1234 };
        let mut buffer = [0u8; AirSpeed::MIN_PAYLOAD_SIZE];
        let _ = airspeed.to_bytes(&mut buffer);
        assert_eq!(buffer, [0x04, 0xD2]);
    }

    #[test]
    fn test_airspeed_from_bytes() {
        let data: [u8; AirSpeed::MIN_PAYLOAD_SIZE] = [0x04, 0xD2];
        let airspeed = AirSpeed::from_bytes(&data).unwrap();
        assert_eq!(airspeed.speed, 1234);
    }

    #[test]
    fn test_airspeed_round_trip() {
        let airspeed = AirSpeed { speed: 5678 };
        let mut buffer = [0u8; AirSpeed::MIN_PAYLOAD_SIZE];
        let _ = airspeed.to_bytes(&mut buffer);
        let round_trip_airspeed = AirSpeed::from_bytes(&buffer).unwrap();
        assert_eq!(airspeed, round_trip_airspeed);
    }

    #[test]
    fn test_airspeed_to_bytes_buffer_too_small() {
        let airspeed = AirSpeed { speed: 1234 };
        let mut buffer = [0u8; 1];
        let result = airspeed.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }

    #[test]
    fn test_airspeed_from_bytes_invalide_size() {
        let data: [u8; 1] = [0x04];
        let result = AirSpeed::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }
}
