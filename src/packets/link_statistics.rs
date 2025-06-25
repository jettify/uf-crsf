#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LinkStatistics {
    pub uplink_rssi_1: u8,
    pub uplink_rssi_2: u8,
    pub uplink_link_quality: u8,
    pub uplink_snr: i8,
    pub active_antenna: u8,
    pub rf_mode: u8,
    pub uplink_tx_power: u8,
    pub downlink_rssi: u8,
    pub downlink_link_quality: u8,
    pub downlink_snr: i8,
}

impl LinkStatistics {
    pub const SERIALIZED_LEN: usize = 10;

    pub fn to_bytes(&self, buffer: &mut [u8; Self::SERIALIZED_LEN]) {
        buffer[0] = self.uplink_rssi_1;
        buffer[1] = self.uplink_rssi_2;
        buffer[2] = self.uplink_link_quality;
        buffer[3] = self.uplink_snr as u8;
        buffer[4] = self.active_antenna;
        buffer[5] = self.rf_mode;
        buffer[6] = self.uplink_tx_power;
        buffer[7] = self.downlink_rssi;
        buffer[8] = self.downlink_link_quality;
        buffer[9] = self.downlink_snr as u8;
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        Self {
            uplink_rssi_1: data[0],
            uplink_rssi_2: data[1],
            uplink_link_quality: data[2],
            uplink_snr: data[3] as i8,
            active_antenna: data[4],
            rf_mode: data[5],
            uplink_tx_power: data[6],
            downlink_rssi: data[7],
            downlink_link_quality: data[8],
            downlink_snr: data[9] as i8,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
