//!   uint8_t rssi_db;        // RSSI (dBm * -1)
//!   uint8_t rssi_percent;   // RSSI in percent
//!   uint8_t link_quality;   // Package success rate / Link quality (%)
//!   int8_t  snr;            // SNR (dB)
//!   uint8_t rf_power_db;    // rf power in dBm
use crate::CrsfParsingError;
use crate::packets::CrsfPacket;
use crate::packets::PacketType;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LinkStatisticsRx {
    pub rssi_db: u8,
    pub rssi_percent: u8,
    pub link_quality: u8,
    pub snr: i8,
    pub rf_power_db: u8,
}

impl CrsfPacket for LinkStatisticsRx {
    const PACKET_TYPE: PacketType = PacketType::LinkStatisticsRx;
    const MIN_PAYLOAD_SIZE: usize = 5;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0] = self.rssi_db;
        buffer[1] = self.rssi_percent;
        buffer[2] = self.link_quality;
        buffer[3] = self.snr as u8;
        buffer[4] = self.rf_power_db;
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Ok(Self {
            rssi_db: data[0],
            rssi_percent: data[1],
            link_quality: data[2],
            snr: data[3] as i8,
            rf_power_db: data[4],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_statistics_rx_to_bytes() {
        let link_statistics_rx = LinkStatisticsRx {
            rssi_db: 100,
            rssi_percent: 75,
            link_quality: 90,
            snr: -10,
            rf_power_db: 20,
        };

        let mut buffer = [0u8; LinkStatisticsRx::MIN_PAYLOAD_SIZE];
        let _ = link_statistics_rx.to_bytes(&mut buffer);

        let expected_bytes: [u8; LinkStatisticsRx::MIN_PAYLOAD_SIZE] = [100, 75, 90, 246, 20];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_link_statistics_rx_from_bytes() {
        let data: [u8; LinkStatisticsRx::MIN_PAYLOAD_SIZE] = [100, 75, 90, 246, 20];

        let link_statistics_rx = LinkStatisticsRx::from_bytes(&data).unwrap();

        assert_eq!(
            link_statistics_rx,
            LinkStatisticsRx {
                rssi_db: 100,
                rssi_percent: 75,
                link_quality: 90,
                snr: -10,
                rf_power_db: 20,
            }
        );
    }

    #[test]
    fn test_link_statistics_rx_round_trip() {
        let link_statistics_rx = LinkStatisticsRx {
            rssi_db: 100,
            rssi_percent: 75,
            link_quality: 90,
            snr: -10,
            rf_power_db: 20,
        };

        let mut buffer = [0u8; LinkStatisticsRx::MIN_PAYLOAD_SIZE];
        link_statistics_rx.to_bytes(&mut buffer).unwrap();

        let round_trip_link_statistics_rx = LinkStatisticsRx::from_bytes(&buffer).unwrap();

        assert_eq!(link_statistics_rx, round_trip_link_statistics_rx);
    }

    #[test]
    fn test_edge_cases() {
        let link_statistics_rx = LinkStatisticsRx {
            rssi_db: 255,
            rssi_percent: 100,
            link_quality: 100,
            snr: -128,
            rf_power_db: 50,
        };

        let mut buffer = [0u8; LinkStatisticsRx::MIN_PAYLOAD_SIZE];
        link_statistics_rx.to_bytes(&mut buffer).unwrap();
        let round_trip_link_statistics_rx = LinkStatisticsRx::from_bytes(&buffer).unwrap();
        assert_eq!(link_statistics_rx, round_trip_link_statistics_rx);
    }
}
