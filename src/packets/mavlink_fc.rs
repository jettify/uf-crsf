use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a `MAVLink` FC packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MavLinkFc {
    pub airspeed: i16,
    /// Vehicle mode flags, defined in MAV_MODE_FLAG enum.
    pub base_mode: u8,
    /// Autopilot-specific flags.
    pub custom_mode: u32,
    /// FC type; defined in MAV_AUTOPILOT enum.
    pub autopilot_type: u8,
    /// Vehicle type; defined in MAV_TYPE enum.
    pub firmware_type: u8,
}

impl CrsfPacket for MavLinkFc {
    const PACKET_TYPE: PacketType = PacketType::MavLinkFc;
    const MIN_PAYLOAD_SIZE: usize = 9;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0..2].copy_from_slice(&self.airspeed.to_be_bytes());
        buffer[2] = self.base_mode;
        buffer[3..7].copy_from_slice(&self.custom_mode.to_be_bytes());
        buffer[7] = self.autopilot_type;
        buffer[8] = self.firmware_type;
        Ok(Self::MIN_PAYLOAD_SIZE)
    }
    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() == Self::MIN_PAYLOAD_SIZE {
            Ok(Self {
                airspeed: i16::from_be_bytes(data[0..2].try_into().unwrap()),
                base_mode: data[2],
                custom_mode: u32::from_be_bytes(data[3..7].try_into().unwrap()),
                autopilot_type: data[7],
                firmware_type: data[8],
            })
        } else {
            Err(CrsfParsingError::InvalidPayloadLength)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mavlink_fc_to_bytes() {
        let mavlink_fc = MavLinkFc {
            airspeed: 12345,
            base_mode: 81,
            custom_mode: 123456789,
            autopilot_type: 3,
            firmware_type: 1,
        };

        let mut buffer = [0u8; MavLinkFc::MIN_PAYLOAD_SIZE];
        let _ = mavlink_fc.to_bytes(&mut buffer);

        let mut expected_bytes = [0u8; MavLinkFc::MIN_PAYLOAD_SIZE];
        expected_bytes[0..2].copy_from_slice(&12345i16.to_be_bytes());
        expected_bytes[2] = 81;
        expected_bytes[3..7].copy_from_slice(&123456789u32.to_be_bytes());
        expected_bytes[7] = 3;
        expected_bytes[8] = 1;

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_mavlink_fc_from_bytes() {
        let mut data = [0u8; MavLinkFc::MIN_PAYLOAD_SIZE];
        data[0..2].copy_from_slice(&12345i16.to_be_bytes());
        data[2] = 81;
        data[3..7].copy_from_slice(&123456789u32.to_be_bytes());
        data[7] = 3;
        data[8] = 1;

        let mavlink_fc = MavLinkFc::from_bytes(&data).unwrap();

        assert_eq!(
            mavlink_fc,
            MavLinkFc {
                airspeed: 12345,
                base_mode: 81,
                custom_mode: 123456789,
                autopilot_type: 3,
                firmware_type: 1,
            }
        );
    }

    #[test]
    fn test_mavlink_fc_round_trip() {
        let mavlink_fc = MavLinkFc {
            airspeed: 5432,
            base_mode: 217,
            custom_mode: 987654321,
            autopilot_type: 12,
            firmware_type: 8,
        };

        let mut buffer = [0u8; MavLinkFc::MIN_PAYLOAD_SIZE];
        mavlink_fc.to_bytes(&mut buffer).unwrap();

        let round_trip_mavlink_fc = MavLinkFc::from_bytes(&buffer).unwrap();

        assert_eq!(mavlink_fc, round_trip_mavlink_fc);
    }

    #[test]
    fn test_edge_cases() {
        let mavlink_fc = MavLinkFc {
            airspeed: i16::MIN,
            base_mode: u8::MAX,
            custom_mode: u32::MAX,
            autopilot_type: u8::MAX,
            firmware_type: u8::MAX,
        };

        let mut buffer = [0u8; MavLinkFc::MIN_PAYLOAD_SIZE];
        mavlink_fc.to_bytes(&mut buffer).unwrap();
        let round_trip_mavlink_fc = MavLinkFc::from_bytes(&buffer).unwrap();
        assert_eq!(mavlink_fc, round_trip_mavlink_fc);

        let mavlink_fc = MavLinkFc {
            airspeed: i16::MAX,
            base_mode: u8::MIN,
            custom_mode: u32::MIN,
            autopilot_type: u8::MIN,
            firmware_type: u8::MIN,
        };

        let mut buffer = [0u8; MavLinkFc::MIN_PAYLOAD_SIZE];
        mavlink_fc.to_bytes(&mut buffer).unwrap();
        let round_trip_mavlink_fc = MavLinkFc::from_bytes(&buffer).unwrap();
        assert_eq!(mavlink_fc, round_trip_mavlink_fc);
    }
}
