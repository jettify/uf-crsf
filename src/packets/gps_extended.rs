//    uint8_t fix_type;       // Current GPS fix quality
//    int16_t n_speed;        // Northward (north = positive) Speed [cm/sec]
//    int16_t e_speed;        // Eastward (east = positive) Speed [cm/sec]
//    int16_t v_speed;        // Vertical (up = positive) Speed [cm/sec]
//    int16_t h_speed_acc;    // Horizontal Speed accuracy cm/sec
//    int16_t track_acc;      // Heading accuracy in degrees scaled with 1e-1 degrees times 10)
//    int16_t alt_ellipsoid;  // Meters Height above GPS Ellipsoid (not MSL)
//    int16_t h_acc;          // horizontal accuracy in cm
//    int16_t v_acc;          // vertical accuracy in cm
//    uint8_t reserved;
//    uint8_t hDOP;           // Horizontal dilution of precision,Dimensionless in nits of.1.
//    uint8_t vDOP;           // vertical dilution of precision, Dimensionless in nits of .1.

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GpsExtended {
    pub fix_type: u8,
    pub n_speed: i16,
    pub e_speed: i16,
    pub v_speed: i16,
    pub h_speed_acc: i16,
    pub track_acc: i16,
    pub alt_ellipsoid: i16,
    pub h_acc: i16,
    pub v_acc: i16,
    pub reserved: u8,
    pub h_dop: u8,
    pub v_dop: u8,
}

impl GpsExtended {
    pub const SERIALIZED_LEN: usize = 20;

    pub fn to_bytes(&self, buffer: &mut [u8; Self::SERIALIZED_LEN]) {
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
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        Self {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_extended_to_bytes() {
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

        let mut buffer = [0u8; GpsExtended::SERIALIZED_LEN];
        gps_extended.to_bytes(&mut buffer);

        let expected_bytes: [u8; GpsExtended::SERIALIZED_LEN] = [
            1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0, 9, 10, 11, 12,
        ];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_gps_extended_from_bytes() {
        let data: [u8; GpsExtended::SERIALIZED_LEN] = [
            1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0, 9, 10, 11, 12,
        ];

        let gps_extended = GpsExtended::from_bytes(&data);

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

        let mut buffer = [0u8; GpsExtended::SERIALIZED_LEN];
        gps_extended.to_bytes(&mut buffer);

        let round_trip_gps_extended = GpsExtended::from_bytes(&buffer);

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

        let mut buffer = [0u8; GpsExtended::SERIALIZED_LEN];
        gps_extended.to_bytes(&mut buffer);
        let round_trip_gps_extended = GpsExtended::from_bytes(&buffer);
        assert_eq!(gps_extended, round_trip_gps_extended);
    }
}
