use crate::CrsfParsingError;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Heartbeat {
    pub origin_address: i16,
}

impl Heartbeat {
    pub const SERIALIZED_LEN: usize = 2;

    pub fn to_bytes(&self, buffer: &mut [u8; Self::SERIALIZED_LEN]) {
        buffer[0..2].copy_from_slice(&self.origin_address.to_be_bytes());
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Result<Self, CrsfParsingError> {
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
        let mut buffer = [0u8; Heartbeat::SERIALIZED_LEN];
        heatbeat.to_bytes(&mut buffer);
        assert_eq!(buffer, [0x04, 0xD2]);
    }

    #[test]
    fn test_heatbeat_from_bytes() {
        let data: [u8; Heartbeat::SERIALIZED_LEN] = [0x04, 0xD2];
        let heatbeat = Heartbeat::from_bytes(&data).unwrap();
        assert_eq!(heatbeat.origin_address, 1234);
    }

    #[test]
    fn test_heatbeat_round_trip() {
        let heatbeat = Heartbeat {
            origin_address: 5678,
        };
        let mut buffer = [0u8; Heartbeat::SERIALIZED_LEN];
        heatbeat.to_bytes(&mut buffer);
        let round_trip_heatbeat = Heartbeat::from_bytes(&buffer).unwrap();
        assert_eq!(heatbeat, round_trip_heatbeat);
    }
}
