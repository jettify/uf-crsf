use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents an Extended GPS packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GpsExtended {
    /// Current GPS fix quality.
    pub fix_type: u8,
    /// Northward speed (north = positive) in cm/s.
    pub n_speed: i16,
    /// Eastward speed (east = positive) in cm/s.
    pub e_speed: i16,
    /// Vertical speed (up = positive) in cm/s.
    pub v_speed: i16,
    /// Horizontal speed accuracy in cm/s.
    pub h_speed_acc: i16,
    /// Heading accuracy in 0.1 degrees.
    pub track_acc: i16,
    /// Height above GPS Ellipsoid (not MSL) in meters.
    pub alt_ellipsoid: i16,
    /// Horizontal accuracy in cm.
    pub h_acc: i16,
    /// Vertical accuracy in cm.
    pub v_acc: i16,
    /// Reserved for future use.
    pub reserved: u8,
    /// Horizontal dilution of precision in 0.1 units.
    pub h_dop: u8,
    /// Vertical dilution of precision in 0.1 units.
    pub v_dop: u8,
}

impl GpsExtended {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fix_type: u8,
        n_speed: i16,
        e_speed: i16,
        v_speed: i16,
        h_speed_acc: i16,
        track_acc: i16,
        alt_ellipsoid: i16,
        h_acc: i16,
        v_acc: i16,
        reserved: u8,
        h_dop: u8,
        v_dop: u8,
    ) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            fix_type,
            n_speed,
            e_speed,
            v_speed,
            h_speed_acc,
            track_acc,
            alt_ellipsoid,
            h_acc,
            v_acc,
            reserved,
            h_dop,
            v_dop,
        })
    }
}

impl CrsfPacket for GpsExtended {
    const PACKET_TYPE: PacketType = PacketType::GpsExtended;
    const MIN_PAYLOAD_SIZE: usize = 4 * size_of::<u8>() + (4 + 4) * size_of::<i16>();

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0] = self.fix_type;
        buffer[1..3].copy_from_slice(&self.n_speed.to_be_bytes());
        buffer[3..5].copy_from_slice(&self.e_speed.to_be_bytes());
        buffer[5..7].copy_from_slice(&self.v_speed.to_be_bytes());
        buffer[7..9].copy_from_slice(&self.h_speed_acc.to_be_bytes());
        buffer[9..11].copy_from_slice(&self.track_acc.to_be_bytes());
        buffer[11..13].copy_from_slice(&self.alt_ellipsoid.to_be_bytes());
        buffer[13..15].copy_from_slice(&self.h_acc.to_be_bytes());
        buffer[15..17].copy_from_slice(&self.v_acc.to_be_bytes());
        buffer[17] = self.reserved;
        buffer[18] = self.h_dop;
        buffer[19] = self.v_dop;
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            fix_type: data[0],
            n_speed: i16::from_be_bytes(data[1..3].try_into().unwrap()),
            e_speed: i16::from_be_bytes(data[3..5].try_into().unwrap()),
            v_speed: i16::from_be_bytes(data[5..7].try_into().unwrap()),
            h_speed_acc: i16::from_be_bytes(data[7..9].try_into().unwrap()),
            track_acc: i16::from_be_bytes(data[9..11].try_into().unwrap()),
            alt_ellipsoid: i16::from_be_bytes(data[11..13].try_into().unwrap()),
            h_acc: i16::from_be_bytes(data[13..15].try_into().unwrap()),
            v_acc: i16::from_be_bytes(data[15..17].try_into().unwrap()),
            reserved: data[17],
            h_dop: data[18],
            v_dop: data[19],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_extended_to_bytes() {
        assert_eq!(GpsExtended::MIN_PAYLOAD_SIZE, 20);
        let gps_extended = GpsExtended {
            fix_type: 1,
            n_speed: 2,
            e_speed: 3,
            v_speed: 4,
            h_speed_acc: 5,
            track_acc: 6,
            alt_ellipsoid: 7,
            h_acc: 8,
            v_acc: 9,
            reserved: 10,
            h_dop: 11,
            v_dop: 12,
        };

        let mut buffer = [0u8; GpsExtended::MIN_PAYLOAD_SIZE];
        gps_extended.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; GpsExtended::MIN_PAYLOAD_SIZE] = [
            1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0, 9, 10, 11, 12,
        ];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_gps_extended_from_bytes() {
        let data: [u8; GpsExtended::MIN_PAYLOAD_SIZE] = [
            1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0, 9, 10, 11, 12,
        ];

        let gps_extended = GpsExtended::from_bytes(&data).unwrap();

        assert_eq!(
            gps_extended,
            GpsExtended {
                fix_type: 1,
                n_speed: 2,
                e_speed: 3,
                v_speed: 4,
                h_speed_acc: 5,
                track_acc: 6,
                alt_ellipsoid: 7,
                h_acc: 8,
                v_acc: 9,
                reserved: 10,
                h_dop: 11,
                v_dop: 12,
            }
        );
    }

    #[test]
    fn test_gps_extended_round_trip() {
        let gps_extended = GpsExtended {
            fix_type: 1,
            n_speed: 2,
            e_speed: 3,
            v_speed: 4,
            h_speed_acc: 5,
            track_acc: 6,
            alt_ellipsoid: 7,
            h_acc: 8,
            v_acc: 9,
            reserved: 10,
            h_dop: 11,
            v_dop: 12,
        };

        let mut buffer = [0u8; GpsExtended::MIN_PAYLOAD_SIZE];
        gps_extended.to_bytes(&mut buffer).unwrap();

        let round_trip_gps_extended = GpsExtended::from_bytes(&buffer).unwrap();

        assert_eq!(gps_extended, round_trip_gps_extended);
    }

    #[test]
    fn test_edge_cases() {
        let gps_extended = GpsExtended {
            fix_type: 255,
            n_speed: -32768,
            e_speed: 32767,
            v_speed: -1,
            h_speed_acc: 0,
            track_acc: 1,
            alt_ellipsoid: -1000,
            h_acc: 1000,
            v_acc: -500,
            reserved: 200,
            h_dop: 100,
            v_dop: 50,
        };

        let mut buffer = [0u8; GpsExtended::MIN_PAYLOAD_SIZE];
        gps_extended.to_bytes(&mut buffer).unwrap();
        let round_trip_gps_extended = GpsExtended::from_bytes(&buffer).unwrap();
        assert_eq!(gps_extended, round_trip_gps_extended);
    }

    #[test]
    fn test_to_bytes_buffer_too_small() {
        let gps_extended = GpsExtended {
            fix_type: 255,
            n_speed: -32768,
            e_speed: 32767,
            v_speed: -1,
            h_speed_acc: 0,
            track_acc: 1,
            alt_ellipsoid: -1000,
            h_acc: 1000,
            v_acc: -500,
            reserved: 200,
            h_dop: 100,
            v_dop: 50,
        };
        let mut buffer: [u8; 14] = [0; 14];
        let result = gps_extended.to_bytes(&mut buffer);
        assert!(matches!(result, Err(CrsfParsingError::BufferOverflow)));
    }

    #[test]
    fn test_from_bytes_invalid_len() {
        let raw_bytes: [u8; 14] = [0; 14];
        let result = GpsExtended::from_bytes(&raw_bytes);
        assert!(matches!(
            result,
            Err(CrsfParsingError::InvalidPayloadLength)
        ));
    }
}
