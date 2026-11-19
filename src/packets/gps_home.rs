use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Home position recorded by the flight controller, typically at arming (type 0x23).
///
/// Uses the same coordinate and altitude encoding as [`super::Gps`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GpsHome {
    /// Latitude in degrees * 10^7.
    pub latitude: i32,
    /// Longitude in degrees * 10^7.
    pub longitude: i32,
    /// Encoded altitude in meters: actual altitude + 1000.
    pub altitude: u16,
}

impl GpsHome {
    pub fn new(latitude: i32, longitude: i32, altitude: u16) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            latitude,
            longitude,
            altitude,
        })
    }
}

impl CrsfPacket for GpsHome {
    const PACKET_TYPE: PacketType = PacketType::GpsHome;
    const MIN_PAYLOAD_SIZE: usize = 10;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Ok(Self {
            latitude: i32::from_be_bytes([data[0], data[1], data[2], data[3]]),
            longitude: i32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            altitude: u16::from_be_bytes([data[8], data[9]]),
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[..4].copy_from_slice(&self.latitude.to_be_bytes());
        buffer[4..8].copy_from_slice(&self.longitude.to_be_bytes());
        buffer[8..10].copy_from_slice(&self.altitude.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_wire_payload() {
        // 1 degree north, 1 degree west, 50 meters above sea level.
        let payload = [0x00, 0x98, 0x96, 0x80, 0xff, 0x67, 0x69, 0x80, 0x04, 0x1a];
        let home = GpsHome::new(10_000_000, -10_000_000, 1050).unwrap();
        assert_eq!(GpsHome::from_bytes(&payload).unwrap(), home);
        let mut buffer = [0xa5; 12];
        assert_eq!(home.to_bytes(&mut buffer).unwrap(), 10);
        assert_eq!(&buffer[..10], &payload);
        assert_eq!(&buffer[10..], &[0xa5; 2]);
    }

    #[test]
    fn preserves_wire_value_limits() {
        for (latitude, longitude, altitude) in [
            (i32::MIN, i32::MAX, 0),
            (i32::MAX, i32::MIN, u16::MAX),
            (0, 0, 1000),
        ] {
            let home = GpsHome::new(latitude, longitude, altitude).unwrap();
            let mut buffer = [0; 10];
            home.to_bytes(&mut buffer).unwrap();
            assert_eq!(GpsHome::from_bytes(&buffer).unwrap(), home);
        }
    }

    #[test]
    fn rejects_short_output_without_writing() {
        let home = GpsHome::new(0, 0, 1000).unwrap();
        for len in 0..10 {
            let mut buffer = [0xa5; 10];
            assert_eq!(
                home.to_bytes(&mut buffer[..len]),
                Err(CrsfParsingError::BufferOverflow),
            );
            assert_eq!(buffer, [0xa5; 10]);
        }
    }
}
