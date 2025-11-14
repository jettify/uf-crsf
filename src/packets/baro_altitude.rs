use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;
use core::f32::consts::E;
use libm::{logf, powf};

/// Represents a Barometric Altitude & Vertical Speed packet.
///
/// This frame allows sending altitude and vertical speed in a bit-efficient way.
/// It combines decimeter-precision altitude with a 32-km range and
/// 3cm/s-precision vertical speed with a 25m/s range into just 3 bytes.
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BaroAltitude {
    /// Packed altitude above start (calibration) point.
    /// See `get_altitude_dm()` for unpacking.
    pub altitude_packed: u16,
    /// Packed vertical speed. See `get_vertical_speed_cm_s()` for unpacking.
    pub vertical_speed_packed: i8,
}

impl BaroAltitude {
    pub fn new(altitude_packed: u16, vertical_speed_packed: i8) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            altitude_packed,
            vertical_speed_packed,
        })
    }
}

impl BaroAltitude {
    /// MSB = 0: altitude is in decimeters - 10000dm offset (so 0 represents -1000m; 10000 represents 0m (starting altitude); 0x7fff represents 2276.7m);
    /// MSB = 1: altitude is in meters. Without any offset.
    pub fn get_altitude_dm(&self) -> i32 {
        if (self.altitude_packed & 0x8000) != 0 {
            (i32::from(self.altitude_packed & 0x7fff)) * 10
        } else {
            (i32::from(self.altitude_packed)) - 10000
        }
    }

    pub fn get_altitude_packed(altitude_dm: i32) -> u16 {
        const ALT_MIN_DM: i32 = 10000;
        const ALT_THRESHOLD_DM: i32 = 0x8000 - ALT_MIN_DM;
        const ALT_MAX_DM: i32 = 0x7ffe * 10 - 5;

        if altitude_dm < -ALT_MIN_DM {
            0
        } else if altitude_dm > ALT_MAX_DM {
            0xfffe
        } else if altitude_dm < ALT_THRESHOLD_DM {
            (altitude_dm + ALT_MIN_DM) as u16
        } else {
            (((altitude_dm + 5) / 10) | 0x8000) as u16
        }
    }

    pub fn get_vertical_speed_packed(vertical_speed_cm_s: i16) -> i8 {
        (logf((f32::from(vertical_speed_cm_s.abs())) / KL + 1.0) / KR
            * (f32::from(vertical_speed_cm_s.signum()))) as i8
    }

    pub fn get_vertical_speed_cm_s(&self) -> i16 {
        ((powf(E, (f32::from(self.vertical_speed_packed.abs())) * KR) - 1.0)
            * KL
            * (f32::from(self.vertical_speed_packed.signum()))) as i16
    }
}

const KL: f32 = 100.0; // linearity constant
const KR: f32 = 0.026; // range constant

impl CrsfPacket for BaroAltitude {
    const PACKET_TYPE: PacketType = PacketType::BaroAltitude;
    const MIN_PAYLOAD_SIZE: usize = size_of::<u16>() + size_of::<i8>();

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0..2].copy_from_slice(&self.altitude_packed.to_be_bytes());
        buffer[2] = self.vertical_speed_packed as u8;
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            altitude_packed: u16::from_be_bytes(
                data[0..2]
                    .try_into()
                    .map_err(|_| CrsfParsingError::InvalidPayloadLength)?,
            ),
            vertical_speed_packed: data[2] as i8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_altitude_packing() {
        assert_eq!(BaroAltitude::get_altitude_packed(-10000), 0);
        assert_eq!(BaroAltitude::get_altitude_packed(22766), 32766);
        assert_eq!(BaroAltitude::get_altitude_packed(-10001), 0);
        assert_eq!(BaroAltitude::get_altitude_packed(327660), 0xfffe);
        assert_eq!(BaroAltitude::get_altitude_packed(0), 10000);
        assert_eq!(BaroAltitude::get_altitude_packed(22767), 0x7FFF);
    }

    #[test]
    fn test_altitude_unpacking() {
        let baro_altitude_dm = BaroAltitude {
            altitude_packed: 0,
            vertical_speed_packed: 0,
        };
        assert_eq!(baro_altitude_dm.get_altitude_dm(), -10000);

        let baro_altitude_m = BaroAltitude {
            altitude_packed: 0x8000,
            vertical_speed_packed: 0,
        };
        assert_eq!(baro_altitude_m.get_altitude_dm(), 0);

        let baro_altitude_dm = BaroAltitude {
            altitude_packed: 10000,
            vertical_speed_packed: 0,
        };
        assert_eq!(baro_altitude_dm.get_altitude_dm(), 0);

        let baro_altitude_dm = BaroAltitude {
            altitude_packed: 0x7fff,
            vertical_speed_packed: 0,
        };
        assert_eq!(baro_altitude_dm.get_altitude_dm(), 22767);
    }

    #[test]
    fn test_vertical_speed_packing() {
        assert_eq!(BaroAltitude::get_vertical_speed_packed(0), 0);
        assert_eq!(BaroAltitude::get_vertical_speed_packed(2500), 125);
        assert_eq!(BaroAltitude::get_vertical_speed_packed(-2500), -125);
    }

    #[test]
    fn test_vertical_speed_unpacking() {
        let baro_altitude = BaroAltitude {
            altitude_packed: 0,
            vertical_speed_packed: 0,
        };
        assert_eq!(baro_altitude.get_vertical_speed_cm_s(), 0);

        let baro_altitude = BaroAltitude {
            altitude_packed: 0,
            vertical_speed_packed: 127,
        };
        assert_eq!(
            (baro_altitude.get_vertical_speed_cm_s() as f32).round(),
            2616.0
        );

        let baro_altitude = BaroAltitude {
            altitude_packed: 0,
            vertical_speed_packed: -127,
        };
        assert_eq!(
            (baro_altitude.get_vertical_speed_cm_s() as f32).round(),
            -2616.0
        );
    }

    #[test]
    fn test_baro_altitude_to_bytes() {
        let baro_altitude = BaroAltitude {
            altitude_packed: 12345,
            vertical_speed_packed: -50,
        };

        let mut buffer = [0u8; BaroAltitude::MIN_PAYLOAD_SIZE];
        baro_altitude.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; BaroAltitude::MIN_PAYLOAD_SIZE] = [0x30, 0x39, 0xce];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_baro_altitude_from_bytes() {
        assert_eq!(BaroAltitude::MIN_PAYLOAD_SIZE, 3);
        let data: [u8; BaroAltitude::MIN_PAYLOAD_SIZE] = [0x30, 0x39, 0xce];

        let baro_altitude = BaroAltitude::from_bytes(&data).unwrap();

        assert_eq!(
            baro_altitude,
            BaroAltitude {
                altitude_packed: 12345,
                vertical_speed_packed: -50,
            }
        );
    }

    #[test]
    fn test_baro_altitude_round_trip() {
        let baro_altitude = BaroAltitude {
            altitude_packed: 12345,
            vertical_speed_packed: -50,
        };

        let mut buffer = [0u8; BaroAltitude::MIN_PAYLOAD_SIZE];
        baro_altitude.to_bytes(&mut buffer).unwrap();

        let round_trip_baro_altitude = BaroAltitude::from_bytes(&buffer).unwrap();

        assert_eq!(baro_altitude, round_trip_baro_altitude);
    }

    #[test]
    fn test_edge_cases() {
        let baro_altitude = BaroAltitude {
            altitude_packed: 65535,
            vertical_speed_packed: -128,
        };

        let mut buffer = [0u8; BaroAltitude::MIN_PAYLOAD_SIZE];
        baro_altitude.to_bytes(&mut buffer).unwrap();
        let round_trip_baro_altitude = BaroAltitude::from_bytes(&buffer).unwrap();
        assert_eq!(baro_altitude, round_trip_baro_altitude);
    }

    #[test]
    fn test_baro_altitude_to_bytes_buffer_too_small() {
        let baro_altitude = BaroAltitude {
            altitude_packed: 12345,
            vertical_speed_packed: -50,
        };
        let mut buffer = [0u8; 2];
        let result = baro_altitude.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }

    #[test]
    fn test_baro_altitude_from_bytes_invalide_size() {
        let data: [u8; 1] = [0x04];
        let result = BaroAltitude::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }
}
