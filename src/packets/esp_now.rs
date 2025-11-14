use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents an ESP-NOW Messages packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EspNow {
    /// Used for Seat Position of the Pilot.
    pub val1: u8,
    /// Used for the Current Pilots Lap.
    pub val2: u8,
    /// 15 characters for the lap time current/split.
    pub val3: [u8; 15],
    /// 15 characters for the lap time current/split.
    pub val4: [u8; 15],
    /// Free text of 20 characters at the bottom of the screen.
    pub free_text: [u8; 20],
}

impl EspNow {
    pub fn new(
        val1: u8,
        val2: u8,
        val3: [u8; 15],
        val4: [u8; 15],
        free_text: [u8; 20],
    ) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            val1,
            val2,
            val3,
            val4,
            free_text,
        })
    }
}

impl CrsfPacket for EspNow {
    const PACKET_TYPE: PacketType = PacketType::EspNow;
    const MIN_PAYLOAD_SIZE: usize =
        2 * size_of::<u8>() + 2 * size_of::<[u8; 15]>() + size_of::<[u8; 20]>();

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0] = self.val1;
        buffer[1] = self.val2;
        buffer[2..17].copy_from_slice(&self.val3);
        buffer[17..32].copy_from_slice(&self.val4);
        buffer[32..52].copy_from_slice(&self.free_text);
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let mut val3 = [0u8; 15];
        val3.copy_from_slice(&data[2..17]);

        let mut val4 = [0u8; 15];
        val4.copy_from_slice(&data[17..32]);

        let mut free_text = [0u8; 20];
        free_text.copy_from_slice(&data[32..52]);

        Ok(Self {
            val1: data[0],
            val2: data[1],
            val3,
            val4,
            free_text,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_esp_now_to_bytes() {
        assert_eq!(EspNow::MIN_PAYLOAD_SIZE, 52);
        let esp_now = EspNow {
            val1: 10,
            val2: 20,
            val3: [65u8; 15],
            val4: [66u8; 15],
            free_text: [67u8; 20],
        };

        let mut buffer = [0u8; EspNow::MIN_PAYLOAD_SIZE];
        let _ = esp_now.to_bytes(&mut buffer);

        let mut expected_bytes = [0u8; EspNow::MIN_PAYLOAD_SIZE];
        expected_bytes[0] = 10;
        expected_bytes[1] = 20;
        expected_bytes[2..17].copy_from_slice(&[65u8; 15]);
        expected_bytes[17..32].copy_from_slice(&[66u8; 15]);
        expected_bytes[32..52].copy_from_slice(&[67u8; 20]);

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_esp_now_from_bytes() {
        let mut data = [0u8; EspNow::MIN_PAYLOAD_SIZE];
        data[0] = 10;
        data[1] = 20;
        data[2..17].copy_from_slice(&[65u8; 15]);
        data[17..32].copy_from_slice(&[66u8; 15]);
        data[32..52].copy_from_slice(&[67u8; 20]);

        let esp_now = EspNow::from_bytes(&data).unwrap();

        assert_eq!(
            esp_now,
            EspNow {
                val1: 10,
                val2: 20,
                val3: [65u8; 15],
                val4: [66u8; 15],
                free_text: [67u8; 20],
            }
        );
    }

    #[test]
    fn test_esp_now_round_trip() {
        let esp_now = EspNow {
            val1: 10,
            val2: 20,
            val3: [65u8; 15],
            val4: [66u8; 15],
            free_text: [67u8; 20],
        };

        let mut buffer = [0u8; EspNow::MIN_PAYLOAD_SIZE];
        esp_now.to_bytes(&mut buffer).unwrap();

        let round_trip_esp_now = EspNow::from_bytes(&buffer).unwrap();

        assert_eq!(esp_now, round_trip_esp_now);
    }

    #[test]
    fn test_esp_now_from_bytes_too_small() {
        let data: [u8; 5] = [0; 5];
        let result = EspNow::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }

    #[test]
    fn test_esp_now_to_bytes_too_small() {
        let esp_now = EspNow {
            val1: 10,
            val2: 20,
            val3: [65u8; 15],
            val4: [66u8; 15],
            free_text: [67u8; 20],
        };
        let mut buffer = [0u8; 8];
        let result = esp_now.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }
}
