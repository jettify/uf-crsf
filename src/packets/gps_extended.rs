use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

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

impl CrsfPacket for GpsExtended {
    const PACKET_TYPE: PacketType = PacketType::GpsExtended;
    const MIN_PAYLOAD_SIZE: usize = 20;

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
}
