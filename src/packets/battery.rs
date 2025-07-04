//    int16_t voltage;        // Voltage (LSB = 10 µV)
//    int16_t current;        // Current (LSB = 10 µA)
//    uint24_t capacity_used; // Capacity used (mAh)
//    uint8_t remaining;      // Battery remaining (percent)

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Battery {
    pub voltage: i16,
    pub current: i16,
    pub capacity_used: u32,
    pub remaining: u8,
}

impl Battery {
    pub const SERIALIZED_LEN: usize = 8;

    pub fn to_bytes(&self, buffer: &mut [u8; Self::SERIALIZED_LEN]) {
        buffer[0..2].copy_from_slice(&self.voltage.to_be_bytes());
        buffer[2..4].copy_from_slice(&self.current.to_be_bytes());
        buffer[4..7].copy_from_slice(&self.capacity_used.to_be_bytes()[1..]); // Take only the last 3 bytes
        buffer[7] = self.remaining;
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        let mut capacity_bytes: [u8; 4] = [0; 4];
        capacity_bytes[1..].copy_from_slice(&data[4..7]);

        Self {
            voltage: i16::from_be_bytes(data[0..2].try_into().unwrap()),
            current: i16::from_be_bytes(data[2..4].try_into().unwrap()),
            capacity_used: u32::from_be_bytes(capacity_bytes),
            remaining: data[7],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_to_bytes() {
        let battery = Battery {
            voltage: 12345,
            current: -1000,
            capacity_used: 1234567,
            remaining: 75,
        };

        let mut buffer = [0u8; Battery::SERIALIZED_LEN];
        battery.to_bytes(&mut buffer);

        let expected_bytes: [u8; Battery::SERIALIZED_LEN] = [
            0x30, 0x39, // Voltage: 12345
            0xfc, 0x18, // Current: -1000
            0x12, 0xd6, 0x87, // Capacity Used: 1234567
            0x4b, // Remaining: 75
        ];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_battery_from_bytes() {
        let data: [u8; Battery::SERIALIZED_LEN] = [
            0x30, 0x39, // Voltage: 12345
            0xfc, 0x18, // Current: -1000
            0x12, 0xd6, 0x87, // Capacity Used: 1234567
            0x4b, // Remaining: 75
        ];

        let battery = Battery::from_bytes(&data);

        assert_eq!(
            battery,
            Battery {
                voltage: 12345,
                current: -1000,
                capacity_used: 1234567,
                remaining: 75,
            }
        );
    }

    #[test]
    fn test_battery_round_trip() {
        let battery = Battery {
            voltage: 12345,
            current: -1000,
            capacity_used: 1234567,
            remaining: 75,
        };

        let mut buffer = [0u8; Battery::SERIALIZED_LEN];
        battery.to_bytes(&mut buffer);

        let round_trip_battery = Battery::from_bytes(&buffer);

        assert_eq!(battery, round_trip_battery);
    }

    #[test]
    fn test_edge_cases() {
        let battery = Battery {
            voltage: -32768,
            current: 32767,
            capacity_used: 16777215, // Max 24-bit value
            remaining: 255,
        };

        let mut buffer = [0u8; Battery::SERIALIZED_LEN];
        battery.to_bytes(&mut buffer);
        let round_trip_battery = Battery::from_bytes(&buffer);
        assert_eq!(battery, round_trip_battery);
    }
}
