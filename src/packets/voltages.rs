use crate::CrsfParsingError;
use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use heapless::Vec;

#[derive(Clone, Debug, PartialEq)]
pub struct Voltages {
    pub voltage_source_id: u8,
    pub voltage_values: Vec<u16, 29>,
}

impl CrsfPacket for Voltages {
    const PACKET_TYPE: PacketType = PacketType::Voltages;
    const MIN_PAYLOAD_SIZE: usize = 3;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0] = self.voltage_source_id;
        let mut i = 1;
        for &voltage in self.voltage_values.iter() {
            let bytes = voltage.to_be_bytes();
            buffer[i..i + 2].copy_from_slice(&bytes);
            i += 2;
        }
        Ok(i)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let voltage_source_id = data[0];
        let voltage_values: Vec<u16, 29> = data[1..]
            .chunks_exact(2)
            .map(|chunk| {
                let bytes = [chunk[0], chunk[1]];
                u16::from_be_bytes(bytes)
            })
            .collect();

        Ok(Self {
            voltage_source_id,
            voltage_values,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voltages_to_bytes() {
        let mut voltage_values = Vec::new();
        voltage_values.push(3850).unwrap();
        voltage_values.push(3900).unwrap();
        let voltages = Voltages {
            voltage_source_id: 0,
            voltage_values,
        };

        let mut buffer = [0u8; 60];
        let len = voltages.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; 5] = [
            0, // Source ID
            0x0f, 0x0a, // 3850
            0x0f, 0x3c, // 3900
        ];

        assert_eq!(len, 5);
        assert_eq!(&buffer[..len], &expected_bytes);
    }

    #[test]
    fn test_voltages_from_bytes() {
        let data: [u8; 5] = [
            0, // Source ID
            0x0f, 0x0a, // 3850
            0x0f, 0x3c, // 3900
        ];

        let voltages = Voltages::from_bytes(&data).unwrap();

        let mut expected_voltage_values: Vec<u16, 29> = Vec::new();
        expected_voltage_values.push(3850).unwrap();
        expected_voltage_values.push(3900).unwrap();
        assert_eq!(voltages.voltage_source_id, 0);
        assert_eq!(voltages.voltage_values, expected_voltage_values);
    }

    #[test]
    fn test_voltages_round_trip() {
        let mut voltage_values = Vec::new();
        voltage_values.push(3700).unwrap();
        voltage_values.push(3800).unwrap();
        let voltages = Voltages {
            voltage_source_id: 1,
            voltage_values,
        };

        let mut buffer = [0u8; 60];
        let len = voltages.to_bytes(&mut buffer).unwrap();

        let round_trip_voltages = Voltages::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(voltages, round_trip_voltages);
    }

    #[test]
    fn test_edge_cases() {
        let mut voltage_values = Vec::new();
        voltage_values.push(0).unwrap();
        voltage_values.push(65535).unwrap(); // max positive
        let voltages = Voltages {
            voltage_source_id: 2,
            voltage_values,
        };

        let mut buffer = [0u8; 29 * 2];
        let len = voltages.to_bytes(&mut buffer).unwrap();
        let round_trip_voltages = Voltages::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(voltages, round_trip_voltages);
    }
}
