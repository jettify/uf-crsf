use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a Link Statistics Repeater packet (frame type `0x15`).
///
/// Uplink is the connection from the repeater to the UAV and downlink the opposite
/// direction. In a repeater setup, `LinkStatistics` (`0x14`) reports the connection
/// from the ground to the repeater. Both packets use the same 10-byte payload layout.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LinkStatisticsRepeater {
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

impl LinkStatisticsRepeater {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        uplink_rssi_1: u8,
        uplink_rssi_2: u8,
        uplink_link_quality: u8,
        uplink_snr: i8,
        active_antenna: u8,
        rf_mode: u8,
        uplink_tx_power: u8,
        downlink_rssi: u8,
        downlink_link_quality: u8,
        downlink_snr: i8,
    ) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            uplink_rssi_1,
            uplink_rssi_2,
            uplink_link_quality,
            uplink_snr,
            active_antenna,
            rf_mode,
            uplink_tx_power,
            downlink_rssi,
            downlink_link_quality,
            downlink_snr,
        })
    }
}

impl CrsfPacket for LinkStatisticsRepeater {
    const PACKET_TYPE: PacketType = PacketType::LinkStatisticsRepeater;
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
        if data.len() >= Self::MIN_PAYLOAD_SIZE {
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
    fn test_wire_layout() {
        let packet = LinkStatisticsRepeater::new(100, 75, 90, -10, 1, 2, 8, 110, 80, -5).unwrap();
        let data = [100, 75, 90, 246, 1, 2, 8, 110, 80, 251];
        assert_eq!(
            LinkStatisticsRepeater::from_bytes(&data),
            Ok(packet.clone())
        );

        let mut buffer = [0xaa; 11];
        assert_eq!(packet.to_bytes(&mut buffer), Ok(10));
        assert_eq!(&buffer[..10], &data);
        assert_eq!(buffer[10], 0xaa);
    }

    #[test]
    fn test_snr_limits() {
        for (uplink_snr, downlink_snr) in [(i8::MIN, i8::MAX), (i8::MAX, i8::MIN)] {
            let data = [
                255,
                0,
                100,
                uplink_snr as u8,
                0,
                0,
                0,
                255,
                0,
                downlink_snr as u8,
            ];
            let packet = LinkStatisticsRepeater::from_bytes(&data).unwrap();
            assert_eq!(packet.uplink_snr, uplink_snr);
            assert_eq!(packet.downlink_snr, downlink_snr);
            let mut buffer = [0; 10];
            assert_eq!(packet.to_bytes(&mut buffer), Ok(10));
            assert_eq!(buffer, data);
        }
    }

    #[test]
    fn test_invalid_payload_lengths() {
        let data = [0; 60];
        for len in 0..LinkStatisticsRepeater::MIN_PAYLOAD_SIZE {
            assert_eq!(
                LinkStatisticsRepeater::from_bytes(&data[..len]),
                Err(CrsfParsingError::InvalidPayloadLength)
            );
        }
    }

    #[test]
    fn test_short_buffers() {
        let packet = LinkStatisticsRepeater::new(100, 75, 90, -10, 1, 2, 8, 110, 80, -5).unwrap();
        for len in 0..LinkStatisticsRepeater::MIN_PAYLOAD_SIZE {
            let mut buffer = [0xaa; 10];
            assert_eq!(
                packet.to_bytes(&mut buffer[..len]),
                Err(CrsfParsingError::BufferOverflow)
            );
            assert_eq!(buffer, [0xaa; 10]);
        }
    }
}
