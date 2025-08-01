use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a GPS packet (type 0x02).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Gps {
    /// Latitude in degrees * 10^7.
    pub latitude: i32,
    /// Longitude in degrees * 10^7.
    pub longitude: i32,
    /// Groundspeed in 0.01 km/h units.
    pub groundspeed: u16,
    /// Heading in 0.01 degree units.
    pub heading: u16,
    /// Altitude with 1000m offset.
    pub altitude: u16,
    /// Number of satellites in view.
    pub satellites: u8,
}

impl CrsfPacket for Gps {
    const PACKET_TYPE: PacketType = PacketType::Gps;
    const MIN_PAYLOAD_SIZE: usize = 15;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        if buffer.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[0..4].copy_from_slice(&self.latitude.to_be_bytes());
        buffer[4..8].copy_from_slice(&self.longitude.to_be_bytes());
        buffer[8..10].copy_from_slice(&self.groundspeed.to_be_bytes());
        buffer[10..12].copy_from_slice(&self.heading.to_be_bytes());
        buffer[12..14].copy_from_slice(&self.altitude.to_be_bytes());
        buffer[14] = self.satellites;

        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            latitude: i32::from_be_bytes(data[0..4].try_into().expect("infallible")),
            longitude: i32::from_be_bytes(data[4..8].try_into().expect("infallible")),
            groundspeed: u16::from_be_bytes(data[8..10].try_into().expect("infallible")),
            heading: u16::from_be_bytes(data[10..12].try_into().expect("infallible")),
            altitude: u16::from_be_bytes(data[12..14].try_into().expect("infallible")),
            satellites: data[14],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::{write_packet_to_buffer, PacketAddress};

    #[test]
    fn test_gps_from_bytes() {
        let gps = Gps {
            latitude: 124108701,
            longitude: -276434195,
            groundspeed: 26,
            heading: 3500,
            altitude: 1050,
            satellites: 15,
        };
        let mut buffer = [0u8; 64];
        let len = write_packet_to_buffer(&mut buffer, PacketAddress::Broadcast, &gps).unwrap();
        let payload = &buffer[3..len - 1];
        let parsed_gps = Gps::from_bytes(payload).unwrap();
        assert_eq!(gps, parsed_gps);
    }

    #[test]
    fn test_gps_to_bytes() {
        let gps = Gps {
            latitude: 124108701,
            longitude: -276434195,
            groundspeed: 26,
            heading: 3500,
            altitude: 1050,
            satellites: 15,
        };

        let mut buffer = [0u8; 15];
        let len = gps.to_bytes(&mut buffer).unwrap();

        let mut expected_buffer = [0u8; 64];
        let expected_len =
            write_packet_to_buffer(&mut expected_buffer, PacketAddress::Broadcast, &gps).unwrap();
        let expected_payload = &expected_buffer[3..expected_len - 1];

        assert_eq!(len, 15);
        assert_eq!(buffer, expected_payload);
    }

    #[test]
    fn test_gps_round_trip() {
        let gps = Gps {
            latitude: 525200000,  // Example: 52.52 degrees
            longitude: 134050000, // Example: 13.405 degrees
            groundspeed: 5000,    // 50.00 km/h
            heading: 18000,       // 180.00 degrees
            altitude: 1100,       // 100m
            satellites: 12,
        };

        let mut buffer: [u8; 15] = [0; 15];
        gps.to_bytes(&mut buffer).unwrap();

        let parsed_gps = Gps::from_bytes(&buffer).unwrap();
        assert_eq!(gps, parsed_gps);
    }

    #[test]
    fn test_from_bytes_invalid_len() {
        let raw_bytes: [u8; 14] = [0; 14];
        let result = Gps::from_bytes(&raw_bytes);
        assert!(matches!(result, Err(CrsfParsingError::InvalidPayloadLength)));
    }

    #[test]
    fn test_to_bytes_buffer_too_small() {
        let gps = Gps {
            latitude: 0,
            longitude: 0,
            groundspeed: 0,
            heading: 0,
            altitude: 0,
            satellites: 0,
        };
        let mut buffer: [u8; 14] = [0; 14];
        let result = gps.to_bytes(&mut buffer);
        assert!(matches!(result, Err(CrsfParsingError::BufferOverflow)));
    }

    #[test]
    fn test_gps_from_hardware_bytes() {
        // Raw packet from hardware: [234, 17, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 235, 0, 26]
        // Payload is the 15 bytes after the type.
        let payload: [u8; 15] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 235, 0];
        let gps = Gps::from_bytes(&payload).unwrap();

        assert_eq!(gps.latitude, 0);
        assert_eq!(gps.longitude, 0);
        assert_eq!(gps.groundspeed, 0);
        assert_eq!(gps.heading, 0);
        assert_eq!(gps.altitude, 1003);
        assert_eq!(gps.satellites, 0);

        // Test round-trip
        let mut buffer: [u8; 15] = [0; 15];
        gps.to_bytes(&mut buffer).unwrap();
        assert_eq!(buffer, payload);
        let parsed_gps = Gps::from_bytes(&buffer).unwrap();
        assert_eq!(gps, parsed_gps);
    }
}
