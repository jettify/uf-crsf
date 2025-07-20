use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a GPS packet.
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
    const MIN_PAYLOAD_SIZE: usize = 16;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[..4].copy_from_slice(&self.latitude.to_be_bytes());
        buffer[4..8].copy_from_slice(&self.longitude.to_be_bytes());
        buffer[8..10].copy_from_slice(&self.groundspeed.to_be_bytes());
        buffer[10..12].copy_from_slice(&self.heading.to_be_bytes());
        buffer[12..14].copy_from_slice(&self.altitude.to_be_bytes());
        buffer[15] = self.satellites;

        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            latitude: i32::from_be_bytes(data[0..4].try_into().unwrap()),
            longitude: i32::from_be_bytes(data[4..8].try_into().unwrap()),
            groundspeed: u16::from_be_bytes(data[8..10].try_into().unwrap()),
            heading: u16::from_be_bytes(data[10..12].try_into().unwrap()),
            altitude: u16::from_be_bytes(data[12..14].try_into().unwrap()),
            satellites: data[15],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps() {
        let raw_bytes: [u8; 16] = [0; 16];
        let data = &raw_bytes[0..16];
        let gps = Gps::from_bytes(data).unwrap();

        assert_eq!(gps.altitude, 0);
        assert_eq!(gps.longitude, 0);
        assert_eq!(gps.groundspeed, 0);
        assert_eq!(gps.heading, 0);
        assert_eq!(gps.altitude, 0);
        assert_eq!(gps.satellites, 0);

        let mut buffer: [u8; 16] = [0; 16];
        gps.to_bytes(&mut buffer).unwrap();
        assert_eq!(&buffer, data);
    }
}
