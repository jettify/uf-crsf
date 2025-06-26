//! GPS Packet
//!
//!    int32_t latitude;       // degree / 10`000`000
//!    int32_t longitude;      // degree / 10`000`000
//!    uint16_t groundspeed;   // km/h / 100
//!    uint16_t heading;       // degree / 100
//!    uint16_t altitude;      // meter - 1000m offset
//!    uint8_t satellites;     // # of sats in view
//!
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Gps {
    pub latitude: i32,    // degree / 10`000`000
    pub longitude: i32,   // degree / 10`000`000
    pub groundspeed: u16, // km/h / 100
    pub heading: u16,     // degree / 100
    pub altitude: u16,    // meter - 1000m offset
    pub satellites: u8,   // # of sats in view
}

impl Gps {
    pub const SERIALIZED_LEN: usize = 16;

    pub fn to_bytes(&self, buffer: &mut [u8; Self::SERIALIZED_LEN]) {
        buffer[..4].copy_from_slice(&self.latitude.to_be_bytes());
        buffer[4..8].copy_from_slice(&self.longitude.to_be_bytes());
        buffer[8..10].copy_from_slice(&self.groundspeed.to_be_bytes());
        buffer[10..12].copy_from_slice(&self.heading.to_be_bytes());
        buffer[12..14].copy_from_slice(&self.altitude.to_be_bytes());
        buffer[15] = self.satellites;
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        Self {
            latitude: i32::from_be_bytes(data[0..4].try_into().unwrap()),
            longitude: i32::from_be_bytes(data[4..8].try_into().unwrap()),
            groundspeed: u16::from_be_bytes(data[8..10].try_into().unwrap()),
            heading: u16::from_be_bytes(data[10..12].try_into().unwrap()),
            altitude: u16::from_be_bytes(data[12..14].try_into().unwrap()),
            satellites: data[15],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps() {
        let raw_bytes: [u8; 16] = [0; 16];
        let data = &raw_bytes[0..16].try_into().unwrap();
        let gps = Gps::from_bytes(data);

        assert_eq!(gps.altitude, 0);
        assert_eq!(gps.longitude, 0);
        assert_eq!(gps.groundspeed, 0);
        assert_eq!(gps.heading, 0);
        assert_eq!(gps.altitude, 0);
        assert_eq!(gps.satellites, 0);

        let mut buffer: [u8; 16] = [0; 16];
        gps.to_bytes(&mut buffer);
        assert_eq!(&buffer, data);
    }
}
