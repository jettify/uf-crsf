use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a Link Statistics packet.
///
/// This packet provides statistics about the connection quality.
/// Uplink is the connection from the ground to the UAV and downlink the opposite direction.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LinkStatistics {
    /// Uplink RSSI Antenna 1 (dBm * -1).
    pub uplink_rssi_1: u8,
    /// Uplink RSSI Antenna 2 (dBm * -1).
    pub uplink_rssi_2: u8,
    /// Uplink package success rate / link quality (%).
    pub uplink_link_quality: u8,
    /// Uplink SNR (dB).
    pub uplink_snr: i8,
    /// The currently active antenna.
    pub active_antenna: u8,
    /// RF profile, e.g., 4fps = 0, 50fps, 150fps.
    pub rf_mode: u8,
    /// Uplink TX power enum {0mW = 0, 10mW, 25mW, 100mW, 500mW, 1000mW, 2000mW, 250mW, 50mW}.
    pub uplink_tx_power: u8,
    /// Downlink RSSI (dBm * -1).
    pub downlink_rssi: u8,
    /// Downlink package success rate / link quality (%).
    pub downlink_link_quality: u8,
    /// Downlink SNR (dB).
    pub downlink_snr: i8,
}

impl CrsfPacket for LinkStatistics {
    const PACKET_TYPE: PacketType = PacketType::LinkStatistics;
    const MIN_PAYLOAD_SIZE: usize = 10;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
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
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() == Self::MIN_PAYLOAD_SIZE {
            Ok(Self {
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
    fn test_link_statistics_to_bytes() {
        let link_statistics = LinkStatistics {
            uplink_rssi_1: 100,
            uplink_rssi_2: 75,
            uplink_link_quality: 90,
            uplink_snr: -10,
            active_antenna: 1,
            rf_mode: 2,
            uplink_tx_power: 20,
            downlink_rssi: 110,
            downlink_link_quality: 80,
            downlink_snr: -5,
        };

        let mut buffer = [0u8; LinkStatistics::MIN_PAYLOAD_SIZE];
        let _ = link_statistics.to_bytes(&mut buffer);

        let expected_bytes: [u8; LinkStatistics::MIN_PAYLOAD_SIZE] =
            [100, 75, 90, 246, 1, 2, 20, 110, 80, 251];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_link_statistics_from_bytes() {
        let data: [u8; LinkStatistics::MIN_PAYLOAD_SIZE] =
            [100, 75, 90, 246, 1, 2, 20, 110, 80, 251];

        let link_statistics = LinkStatistics::from_bytes(&data).unwrap();

        assert_eq!(
            link_statistics,
            LinkStatistics {
                uplink_rssi_1: 100,
                uplink_rssi_2: 75,
                uplink_link_quality: 90,
                uplink_snr: -10,
                active_antenna: 1,
                rf_mode: 2,
                uplink_tx_power: 20,
                downlink_rssi: 110,
                downlink_link_quality: 80,
                downlink_snr: -5,
            }
        );
    }

    #[test]
    fn test_link_statistics_round_trip() {
        let link_statistics = LinkStatistics {
            uplink_rssi_1: 100,
            uplink_rssi_2: 75,
            uplink_link_quality: 90,
            uplink_snr: -10,
            active_antenna: 1,
            rf_mode: 2,
            uplink_tx_power: 20,
            downlink_rssi: 110,
            downlink_link_quality: 80,
            downlink_snr: -5,
        };

        let mut buffer = [0u8; LinkStatistics::MIN_PAYLOAD_SIZE];
        link_statistics.to_bytes(&mut buffer).unwrap();

        let round_trip_link_statistics = LinkStatistics::from_bytes(&buffer).unwrap();

        assert_eq!(link_statistics, round_trip_link_statistics);
    }

    #[test]
    fn test_edge_cases() {
        let link_statistics = LinkStatistics {
            uplink_rssi_1: 255,
            uplink_rssi_2: 100,
            uplink_link_quality: 100,
            uplink_snr: -128,
            active_antenna: 3,
            rf_mode: 4,
            uplink_tx_power: 50,
            downlink_rssi: 200,
            downlink_link_quality: 90,
            downlink_snr: 127,
        };

        let mut buffer = [0u8; LinkStatistics::MIN_PAYLOAD_SIZE];
        link_statistics.to_bytes(&mut buffer).unwrap();
        let round_trip_link_statistics = LinkStatistics::from_bytes(&buffer).unwrap();
        assert_eq!(link_statistics, round_trip_link_statistics);
    }
}
