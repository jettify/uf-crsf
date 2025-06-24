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

/// Represents a RcChannelsPacked packet
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RcChannelsPacked(pub [u16; 16]);

impl RcChannelsPacked {
    pub const SERIALIZED_LEN: usize = 22;

    pub fn to_bytes(&self, data: &mut [u8; Self::SERIALIZED_LEN]) {
        let ch = &self.0;
        data[0] = (ch[0]) as u8;
        data[1] = (ch[0] >> 8 | ch[1] << 3) as u8;
        data[2] = (ch[1] >> 5 | ch[2] << 6) as u8;
        data[3] = (ch[2] >> 2) as u8;
        data[4] = (ch[2] >> 10 | ch[3] << 1) as u8;
        data[5] = (ch[3] >> 7 | ch[4] << 4) as u8;
        data[6] = (ch[4] >> 4 | ch[5] << 7) as u8;
        data[7] = (ch[5] >> 1) as u8;
        data[8] = (ch[5] >> 9 | ch[6] << 2) as u8;
        data[9] = (ch[6] >> 6 | ch[7] << 5) as u8;
        data[10] = (ch[7] >> 3) as u8;
        data[11] = (ch[8]) as u8;
        data[12] = (ch[8] >> 8 | ch[9] << 3) as u8;
        data[13] = (ch[9] >> 5 | ch[10] << 6) as u8;
        data[14] = (ch[10] >> 2) as u8;
        data[15] = (ch[10] >> 10 | ch[11] << 1) as u8;
        data[16] = (ch[11] >> 7 | ch[12] << 4) as u8;
        data[17] = (ch[12] >> 4 | ch[13] << 7) as u8;
        data[18] = (ch[13] >> 1) as u8;
        data[19] = (ch[13] >> 9 | ch[14] << 2) as u8;
        data[20] = (ch[14] >> 6 | ch[15] << 5) as u8;
        data[21] = (ch[15] >> 3) as u8;
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        let data: [u16; Self::SERIALIZED_LEN] = core::array::from_fn(|i| data[i] as u16);
        const MASK_11BIT: u16 = 0x07FF;
        let mut ch = [MASK_11BIT; 16];
        ch[0] &= data[0] | (data[1] << 8);
        ch[1] &= (data[1] >> 3) | (data[2] << 5);
        ch[2] &= (data[2] >> 6) | (data[3] << 2) | (data[4] << 10);
        ch[3] &= (data[4] >> 1) | (data[5] << 7);
        ch[4] &= (data[5] >> 4) | (data[6] << 4);
        ch[5] &= (data[6] >> 7) | (data[7] << 1) | (data[8] << 9);
        ch[6] &= (data[8] >> 2) | (data[9] << 6);
        ch[7] &= (data[9] >> 5) | (data[10] << 3);
        ch[8] &= data[11] | (data[12] << 8);
        ch[9] &= (data[12] >> 3) | (data[13] << 5);
        ch[10] &= (data[13] >> 6) | (data[14] << 2) | (data[15] << 10);
        ch[11] &= (data[15] >> 1) | (data[16] << 7);
        ch[12] &= (data[16] >> 4) | (data[17] << 4);
        ch[13] &= (data[17] >> 7) | (data[18] << 1) | (data[19] << 9);
        ch[14] &= (data[19] >> 2) | (data[20] << 6);
        ch[15] &= (data[20] >> 5) | (data[21] << 3);

        RcChannelsPacked(ch)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VariometerSensor {
    pub v_speed: i16, // Vertical speed cm/s
}

impl VariometerSensor {
    pub const SERIALIZED_LEN: usize = 2;

    pub fn to_bytes(&self, buffer: &mut [u8; Self::SERIALIZED_LEN]) {
        buffer[..2].copy_from_slice(&self.v_speed.to_be_bytes());
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        Self {
            v_speed: i16::from_be_bytes([data[0], data[1]]),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::LinkStatistics;
    use super::*;
    #[test]
    fn test_basic_link_stat() {
        let raw_bytes: [u8; 14] = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
        let data = &raw_bytes[3..13].try_into().unwrap();
        let ls = LinkStatistics::from_bytes(data);
        let mut buffer: [u8; 10] = [0; 10];
        ls.to_bytes(&mut buffer);
        assert_eq!(ls.uplink_rssi_1, 16);
        assert_eq!(&buffer, data);
    }

    #[test]
    fn test_rc_channeld() {
        let raw_bytes: [u8; 25] = [
            0xC8, 24, 0x16, 0x03, 0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01,
            0x0B, 0xF8, 0xC0, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 252,
        ];
        let data = &raw_bytes[3..25].try_into().unwrap();
        let rc = RcChannelsPacked::from_bytes(data);
        let mut buffer: [u8; 22] = [0; 22];
        rc.to_bytes(&mut buffer);
        assert_eq!(&buffer, data);
    }
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
    #[test]
    fn test_vario() {
        let raw_bytes: [u8; 2] = [0; 2];
        let data = &raw_bytes[0..2].try_into().unwrap();

        let vario = VariometerSensor::from_bytes(data);
        assert_eq!(vario.v_speed, 0);

        let mut buffer: [u8; 2] = [0; 2];
        vario.to_bytes(&mut buffer);
        assert_eq!(&buffer, data);
    }
}
