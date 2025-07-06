use crate::CrsfParsingError;
use crate::packets::CrsfPacket;
use crate::packets::PacketType;

use heapless::Vec;

#[derive(Clone, Debug, PartialEq)]
pub struct Temp {
    pub temp_source_id: u8,
    pub temperatures: Vec<i16, 20>,
}

impl CrsfPacket for Temp {
    const PACKET_TYPE: PacketType = PacketType::Temp;
    const MIN_PAYLOAD_SIZE: usize = 3;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        buffer[0] = self.temp_source_id;
        let mut i = 1;
        for &temp in self.temperatures.iter() {
            let bytes = temp.to_be_bytes();
            buffer[i..i + 2].copy_from_slice(&bytes);
            i += 2;
        }
        Ok(i)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let temp_source_id = data[0];
        let temperatures: Vec<i16, 20> = data[1..]
            .chunks_exact(2)
            .map(|chunk| {
                let bytes = [chunk[0], chunk[1]];
                i16::from_be_bytes(bytes)
            })
            .collect();

        Ok(Self {
            temp_source_id,
            temperatures,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_to_bytes() {
        let mut temperatures = Vec::new();
        temperatures.push(250).unwrap();
        temperatures.push(-50).unwrap();
        let temp = Temp {
            temp_source_id: 1,
            temperatures,
        };

        let mut buffer = [0u8; 60];
        let len = temp.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; 5] = [
            1, // Source ID
            0x00, 0xfa, // 250
            0xff, 0xce, // -50
        ];

        assert_eq!(len, 5);
        assert_eq!(&buffer[..len], &expected_bytes);
    }

    #[test]
    fn test_temp_from_bytes() {
        let data: [u8; 5] = [
            1, // Source ID
            0x00, 0xfa, // 250
            0xff, 0xce, // -50
        ];

        let temp = Temp::from_bytes(&data).unwrap();

        let mut expected_temperatures: Vec<i16, 20> = Vec::new();
        expected_temperatures.push(250).unwrap();
        expected_temperatures.push(-50).unwrap();
        assert_eq!(temp.temp_source_id, 1);
        assert_eq!(temp.temperatures, expected_temperatures);
    }

    #[test]
    fn test_temp_round_trip() {
        let mut temperatures = Vec::new();
        temperatures.push(1234).unwrap();
        temperatures.push(-5678).unwrap();
        let temp = Temp {
            temp_source_id: 2,
            temperatures,
        };

        let mut buffer = [0u8; 60];
        let len = temp.to_bytes(&mut buffer).unwrap();

        let round_trip_temp = Temp::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(temp, round_trip_temp);
    }

    #[test]
    fn test_edge_cases() {
        let mut temperatures = Vec::new();
        temperatures.push(0).unwrap();
        temperatures.push(32767).unwrap(); // max positive
        temperatures.push(-32768).unwrap(); // min negative
        let temp = Temp {
            temp_source_id: 3,
            temperatures,
        };

        let mut buffer = [0u8; 60];
        let len = temp.to_bytes(&mut buffer).unwrap();
        let round_trip_temp = Temp::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(temp, round_trip_temp);
    }
}
