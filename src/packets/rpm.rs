use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;
use heapless::Vec;

#[derive(Clone, Debug, PartialEq)]
pub struct Rpm {
    pub rpm_source_id: u8,
    pub rpm_values: Vec<i32, 19>,
}

impl CrsfPacket for Rpm {
    const PACKET_TYPE: PacketType = PacketType::Rpm;
    const MIN_PAYLOAD_SIZE: usize = 3;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        buffer[0] = self.rpm_source_id;
        let mut i = 1;
        for &rpm in self.rpm_values.iter() {
            let bytes = rpm.to_be_bytes();
            buffer[i..i + 3].copy_from_slice(&bytes[1..4]);
            i += 3;
        }
        Ok(i)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let rpm_source_id = data[0];
        let rpm_values: Vec<i32, 19> = data[1..]
            .chunks_exact(3)
            .map(|chunk| {
                let mut bytes = [0; 4];
                bytes[1..4].copy_from_slice(chunk);
                let rpm = i32::from_be_bytes(bytes);
                // Sign extend the 24-bit value
                (rpm << 8) >> 8
            })
            .collect();

        Ok(Self {
            rpm_source_id,
            rpm_values,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpm_to_bytes() {
        let mut rpm_values = Vec::new();
        rpm_values.push(1000).unwrap();
        rpm_values.push(-2000).unwrap();
        let rpm = Rpm {
            rpm_source_id: 1,
            rpm_values,
        };

        let mut buffer = [0u8; 60];
        let len = rpm.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; 7] = [
            1, // Source ID
            0x00, 0x03, 0xe8, // 1000
            0xff, 0xf8, 0x30, // -2000
        ];

        assert_eq!(len, 7);
        assert_eq!(&buffer[..len], &expected_bytes);
    }

    #[test]
    fn test_rpm_from_bytes() {
        let data: [u8; 7] = [
            1, // Source ID
            0x00, 0x03, 0xe8, // 1000
            0xff, 0xf8, 0x30, // -2000
        ];

        let rpm = Rpm::from_bytes(&data).unwrap();

        let mut expected_rpm_values: Vec<i32, 19> = Vec::new();
        expected_rpm_values.push(1000).unwrap();
        expected_rpm_values.push(-2000).unwrap();
        assert_eq!(rpm.rpm_source_id, 1);
        assert_eq!(rpm.rpm_values, expected_rpm_values);
    }

    #[test]
    fn test_rpm_round_trip() {
        let mut rpm_values = Vec::new();
        rpm_values.push(123456).unwrap();
        rpm_values.push(-654321).unwrap();
        let rpm = Rpm {
            rpm_source_id: 2,
            rpm_values,
        };

        let mut buffer = [0u8; 60];
        let len = rpm.to_bytes(&mut buffer).unwrap();

        let round_trip_rpm = Rpm::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(rpm, round_trip_rpm);
    }

    #[test]
    fn test_edge_cases() {
        let mut rpm_values = Vec::new();
        rpm_values.push(0).unwrap();
        rpm_values.push(8388607).unwrap(); // max positive
        rpm_values.push(-8388608).unwrap(); // min negative
        let rpm = Rpm {
            rpm_source_id: 3,
            rpm_values,
        };

        let mut buffer = [0u8; 60];
        let len = rpm.to_bytes(&mut buffer).unwrap();
        let round_trip_rpm = Rpm::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(rpm, round_trip_rpm);
    }
}
