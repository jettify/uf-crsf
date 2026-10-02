use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a Battery Sensor packet.
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Battery {
    /// Voltage in 0.1 V units (e.g., 120 represents 12 V).
    pub voltage: i16,
    /// Current in 0.1 A units (e.g., 15 represents 1.5 A).
    pub current: i16,
    /// Capacity used (mAh). This is a 24-bit value.
    pub capacity_used: u32,
    /// Battery remaining (percent).
    pub remaining: u8,
    /// Sensor ID, defaulting to zero when absent from the payload.
    /// A zero ID is omitted when encoding for compatibility with legacy receivers.
    pub id: u8,
}

impl Battery {
    pub fn new(
        voltage: i16,
        current: i16,
        capacity_used: u32,
        remaining: u8,
    ) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            voltage,
            current,
            capacity_used,
            remaining,
            id: 0,
        })
    }
}

impl CrsfPacket for Battery {
    const PACKET_TYPE: PacketType = PacketType::BatterySensor;
    // 24 bit (3 bytes) unpacked into u32 (4 bytes)
    const MIN_PAYLOAD_SIZE: usize = 2 * size_of::<i16>() + 3 + size_of::<u8>();

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let payload_size = Self::MIN_PAYLOAD_SIZE + usize::from(self.id != 0);
        if buffer.len() < payload_size {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[0..2].copy_from_slice(&self.voltage.to_be_bytes());
        buffer[2..4].copy_from_slice(&self.current.to_be_bytes());
        // Take only the last 3 bytes
        buffer[4..7].copy_from_slice(&self.capacity_used.to_be_bytes()[1..]);
        buffer[7] = self.remaining;
        if self.id != 0 {
            buffer[8] = self.id;
        }
        Ok(payload_size)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let mut capacity_bytes: [u8; 4] = [0; 4];
        capacity_bytes[1..].copy_from_slice(&data[4..7]);

        Ok(Self {
            voltage: i16::from_be_bytes(
                data[0..2]
                    .try_into()
                    .map_err(|_| CrsfParsingError::InvalidPayloadLength)?,
            ),
            current: i16::from_be_bytes(
                data[2..4]
                    .try_into()
                    .map_err(|_| CrsfParsingError::InvalidPayloadLength)?,
            ),
            capacity_used: u32::from_be_bytes(capacity_bytes),
            remaining: data[7],
            id: data.get(8).copied().unwrap_or(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_new() {
        let battery = Battery::new(12345, -1000, 1234567, 75).unwrap();
        assert_eq!(battery.voltage, 12345);
        assert_eq!(battery.current, -1000);
        assert_eq!(battery.capacity_used, 1234567);
        assert_eq!(battery.remaining, 75);
        assert_eq!(battery.id, 0);
    }

    #[test]
    fn test_battery_to_bytes() {
        assert_eq!(Battery::MIN_PAYLOAD_SIZE, 8);
        let battery = Battery::new(12345, -1000, 1234567, 75).unwrap();

        let mut buffer = [0u8; Battery::MIN_PAYLOAD_SIZE];
        battery.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; Battery::MIN_PAYLOAD_SIZE] =
            [0x30, 0x39, 0xfc, 0x18, 0x12, 0xd6, 0x87, 0x4b];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_battery_from_bytes() {
        let data: [u8; Battery::MIN_PAYLOAD_SIZE] =
            [0x30, 0x39, 0xfc, 0x18, 0x12, 0xd6, 0x87, 0x4b];

        let battery = Battery::from_bytes(&data).unwrap();

        assert_eq!(
            battery,
            Battery {
                voltage: 12345,
                current: -1000,
                capacity_used: 1234567,
                remaining: 75,
                id: 0,
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
            id: 0,
        };

        let mut buffer = [0u8; Battery::MIN_PAYLOAD_SIZE];
        battery.to_bytes(&mut buffer).unwrap();

        let round_trip_battery = Battery::from_bytes(&buffer).unwrap();

        assert_eq!(battery, round_trip_battery);
    }

    #[test]
    fn test_edge_cases() {
        let battery = Battery {
            voltage: -32768,
            current: 32767,
            capacity_used: 16777215, // Max 24-bit value
            remaining: 255,
            id: 0,
        };

        let mut buffer = [0u8; Battery::MIN_PAYLOAD_SIZE];
        battery.to_bytes(&mut buffer).unwrap();
        let round_trip_battery = Battery::from_bytes(&buffer).unwrap();
        assert_eq!(battery, round_trip_battery);
    }

    #[test]
    fn test_battery_to_bytes_buffer_too_small() {
        let battery = Battery {
            voltage: 12345,
            current: -1000,
            capacity_used: 1234567,
            remaining: 75,
            id: 0,
        };

        let mut buffer = [0u8; 5];
        let result = battery.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }

    #[test]
    fn test_battery_from_bytes_invalid_size() {
        let data = [0u8; Battery::MIN_PAYLOAD_SIZE];
        for len in 0..Battery::MIN_PAYLOAD_SIZE {
            assert_eq!(
                Battery::from_bytes(&data[..len]),
                Err(CrsfParsingError::InvalidPayloadLength)
            );
        }
    }

    #[test]
    fn test_battery_sensor_id_and_extensions() {
        let data = [0, 120, 0, 15, 0x12, 0xd6, 0x87, 75, 255, 0xab, 0xcd];
        let battery = Battery::from_bytes(&data).unwrap();
        assert_eq!(battery.voltage, 120); // 12 V
        assert_eq!(battery.current, 15); // 1.5 A
        assert_eq!(battery.capacity_used, 1234567);
        assert_eq!(battery.remaining, 75);
        assert_eq!(battery.id, 255);

        let mut buffer = [0u8; 9];
        assert_eq!(battery.to_bytes(&mut buffer), Ok(9));
        assert_eq!(buffer, data[..9]);
        assert_eq!(Battery::from_bytes(&buffer).unwrap(), battery);

        let mut short_buffer = [0xa5; 8];
        assert_eq!(
            battery.to_bytes(&mut short_buffer),
            Err(CrsfParsingError::BufferOverflow)
        );
        assert_eq!(short_buffer, [0xa5; 8]);
    }

    #[test]
    fn test_explicit_zero_sensor_id_encodes_as_legacy() {
        let data = [0, 120, 0, 15, 0, 0, 0, 75, 0];
        let battery = Battery::from_bytes(&data).unwrap();
        assert_eq!(battery.id, 0);
        let mut buffer = [0xa5; 9];
        assert_eq!(battery.to_bytes(&mut buffer), Ok(8));
        assert_eq!(buffer[..8], data[..8]);
        assert_eq!(buffer[8], 0xa5);
    }
}
