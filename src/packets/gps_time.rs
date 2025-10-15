use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a GPS Time packet.
///
/// This frame is needed for synchronization with a GPS time pulse.
/// The maximum offset of time is +/-10ms.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GpsTime {
    pub year: i16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
}

impl CrsfPacket for GpsTime {
    const PACKET_TYPE: PacketType = PacketType::GpsTime;
    const MIN_PAYLOAD_SIZE: usize = size_of::<i16>() + 5 * size_of::<u8>() + size_of::<u16>();

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0..2].copy_from_slice(&self.year.to_be_bytes());
        buffer[2] = self.month;
        buffer[3] = self.day;
        buffer[4] = self.hour;
        buffer[5] = self.minute;
        buffer[6] = self.second;
        buffer[7..9].copy_from_slice(&self.millisecond.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            year: i16::from_be_bytes(data[0..2].try_into().unwrap()),
            month: data[2],
            day: data[3],
            hour: data[4],
            minute: data[5],
            second: data[6],
            millisecond: u16::from_be_bytes(data[7..9].try_into().unwrap()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_time_to_bytes() {
        assert_eq!(GpsTime::MIN_PAYLOAD_SIZE, 9);
        let gps_time = GpsTime {
            year: 2024,
            month: 10,
            day: 27,
            hour: 12,
            minute: 34,
            second: 56,
            millisecond: 789,
        };

        let mut buffer = [0u8; GpsTime::MIN_PAYLOAD_SIZE];
        gps_time.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; GpsTime::MIN_PAYLOAD_SIZE] =
            [0x07, 0xe8, 0x0a, 0x1b, 0x0c, 0x22, 0x38, 0x03, 0x15];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_gps_time_from_bytes() {
        let data: [u8; GpsTime::MIN_PAYLOAD_SIZE] =
            [0x07, 0xe8, 0x0a, 0x1b, 0x0c, 0x22, 0x38, 0x03, 0x15];

        let gps_time = GpsTime::from_bytes(&data).unwrap();

        assert_eq!(
            gps_time,
            GpsTime {
                year: 2024,
                month: 10,
                day: 27,
                hour: 12,
                minute: 34,
                second: 56,
                millisecond: 789,
            }
        );
    }

    #[test]
    fn test_gps_time_round_trip() {
        let gps_time = GpsTime {
            year: 2024,
            month: 10,
            day: 27,
            hour: 12,
            minute: 34,
            second: 56,
            millisecond: 789,
        };

        let mut buffer = [0u8; GpsTime::MIN_PAYLOAD_SIZE];
        gps_time.to_bytes(&mut buffer).unwrap();

        let round_trip_gps_time = GpsTime::from_bytes(&buffer).unwrap();

        assert_eq!(gps_time, round_trip_gps_time);
    }

    #[test]
    fn test_edge_cases() {
        let gps_time = GpsTime {
            year: -1,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 65535,
        };

        let mut buffer = [0u8; GpsTime::MIN_PAYLOAD_SIZE];
        gps_time.to_bytes(&mut buffer).unwrap();
        let round_trip_gps_time = GpsTime::from_bytes(&buffer).unwrap();
        assert_eq!(gps_time, round_trip_gps_time);
    }

    #[test]
    fn test_to_bytes_buffer_too_small() {
        let gps_time = GpsTime {
            year: -1,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 65535,
        };
        let mut buffer: [u8; 8] = [0; 8];
        let result = gps_time.to_bytes(&mut buffer);
        assert!(matches!(result, Err(CrsfParsingError::BufferOverflow)));
    }

    #[test]
    fn test_from_bytes_too_small() {
        let data: [u8; 8] = [0; 8];
        let result = GpsTime::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }
}
