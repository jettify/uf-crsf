use crate::CrsfParsingError;
use crate::packets::CrsfPacket;
use crate::packets::PacketType;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AirSpeed {
    pub speed: u16,
}

impl CrsfPacket for AirSpeed {
    const PACKET_TYPE: PacketType = PacketType::AirSpeed;
    const MIN_PAYLOAD_SIZE: usize = 2;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0..2].copy_from_slice(&self.speed.to_be_bytes());
        return Ok(Self::MIN_PAYLOAD_SIZE);
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            speed: u16::from_be_bytes(
                data[0..2]
                    .try_into()
                    .map_err(|_| CrsfParsingError::InvalidPayloadLength)?,
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airspeed_to_bytes() {
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
}
