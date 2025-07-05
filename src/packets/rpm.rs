use heapless::Vec;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Rpm {
    pub rpm_source_id: u8,
    pub rpm_values: Vec<i32, 19>,
}

impl Rpm {
    pub const MAX_LEN: usize = 1 + 19 * 3;

    pub fn to_bytes(&self, buffer: &mut [u8]) -> usize {
        buffer[0] = self.rpm_source_id;
        let mut i = 1;
        for &rpm in self.rpm_values.iter() {
            let bytes = rpm.to_be_bytes();
            buffer[i..i + 3].copy_from_slice(&bytes[1..4]);
            i += 3;
        }
        i
    }

    pub fn from_bytes(data: &[u8]) -> Self {
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

        Self {
            rpm_source_id,
            rpm_values,
        }
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

        let mut buffer = [0u8; Rpm::MAX_LEN];
        let len = rpm.to_bytes(&mut buffer);

        let expected_bytes: [u8; 7] = [
            1,    // Source ID
            0x00, 0x03, 0xe8, // 1000
            0xff, 0xf8, 0x30, // -2000
        ];

        assert_eq!(len, 7);
        assert_eq!(&buffer[..len], &expected_bytes);
    }

    #[test]
    fn test_rpm_from_bytes() {
        let data: [u8; 7] = [
            1,    // Source ID
            0x00, 0x03, 0xe8, // 1000
            0xff, 0xf8, 0x30, // -2000
        ];

        let rpm = Rpm::from_bytes(&data);

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

        let mut buffer = [0u8; Rpm::MAX_LEN];
        let len = rpm.to_bytes(&mut buffer);

        let round_trip_rpm = Rpm::from_bytes(&buffer[..len]);

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

        let mut buffer = [0u8; Rpm::MAX_LEN];
        let len = rpm.to_bytes(&mut buffer);
        let round_trip_rpm = Rpm::from_bytes(&buffer[..len]);
        assert_eq!(rpm, round_trip_rpm);
    }
}
