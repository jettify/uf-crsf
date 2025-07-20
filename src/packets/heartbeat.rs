use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a Heartbeat packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Heartbeat {
    /// Origin device address.
    pub origin_address: i16,
}

impl CrsfPacket for Heartbeat {
    const MIN_PAYLOAD_SIZE: usize = 2;
    const PACKET_TYPE: PacketType = PacketType::Heartbeat;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        buffer[0..2].copy_from_slice(&self.origin_address.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Ok(Self {
            origin_address: i16::from_be_bytes(
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
    fn test_heatbeat_to_bytes() {
        let heatbeat = Heartbeat {
            origin_address: 1234,
        };
        let mut buffer = [0u8; Heartbeat::MIN_PAYLOAD_SIZE];
        heatbeat.to_bytes(&mut buffer).unwrap();
        assert_eq!(buffer, [0x04, 0xD2]);
    }

    #[test]
    fn test_heatbeat_from_bytes() {
        let data: [u8; Heartbeat::MIN_PAYLOAD_SIZE] = [0x04, 0xD2];
        let heatbeat = Heartbeat::from_bytes(&data).unwrap();
        assert_eq!(heatbeat.origin_address, 1234);
    }

    #[test]
    fn test_heatbeat_round_trip() {
        let heatbeat = Heartbeat {
            origin_address: 5678,
        };
        let mut buffer = [0u8; Heartbeat::MIN_PAYLOAD_SIZE];
        heatbeat.to_bytes(&mut buffer).unwrap();
        let round_trip_heatbeat = Heartbeat::from_bytes(&buffer).unwrap();
        assert_eq!(heatbeat, round_trip_heatbeat);
    }
}
