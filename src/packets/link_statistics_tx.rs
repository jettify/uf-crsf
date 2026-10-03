use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Reports the receiver (air side) uplink reception statistics.
///
/// Supports both the original 6-byte payload and the extended 8-byte payload.
/// The antenna extension fields must either both be present or both be absent.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LinkStatisticsTx {
    /// Antenna 1 RSSI (dBm * -1). Older implementations may report combined RSSI.
    pub rssi_db: u8,
    /// RSSI in percent.
    pub rssi_percent: u8,
    /// Package success rate / Link quality (%).
    pub link_quality: u8,
    /// SNR (dB).
    pub snr: i8,
    /// Aircraft receiver RF power in dBm.
    pub rf_power_db: u8,
    /// RF frames per second (fps / 10).
    pub fps: u8,
    /// Antenna 2 RSSI (dBm * -1); `Some(0)` means absent or disabled.
    /// `None` means the field is unavailable in the legacy payload.
    pub rssi_ant2_db: Option<u8>,
    /// Best receive antenna: 0 = antenna 1, 1 = antenna 2.
    /// `None` means the field is unavailable in the legacy payload.
    pub active_antenna: Option<u8>,
}

impl LinkStatisticsTx {
    /// Payload size with the antenna extension fields present.
    pub const EXTENDED_PAYLOAD_SIZE: usize = 8;

    /// Creates a legacy packet with unavailable antenna extension fields.
    /// Set both extension fields to `Some` to serialize an extended payload.
    pub fn new(
        rssi_db: u8,
        rssi_percent: u8,
        link_quality: u8,
        snr: i8,
        rf_power_db: u8,
        fps: u8,
    ) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            rssi_db,
            rssi_percent,
            link_quality,
            snr,
            rf_power_db,
            fps,
            rssi_ant2_db: None,
            active_antenna: None,
        })
    }
}

impl CrsfPacket for LinkStatisticsTx {
    const PACKET_TYPE: PacketType = PacketType::LinkStatisticsTx;
    const MIN_PAYLOAD_SIZE: usize = 6;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let payload_size = match (self.rssi_ant2_db, self.active_antenna) {
            (None, None) => Self::MIN_PAYLOAD_SIZE,
            (Some(_), Some(_)) => Self::EXTENDED_PAYLOAD_SIZE,
            _ => return Err(CrsfParsingError::InvalidPayload),
        };
        if buffer.len() < payload_size {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[0] = self.rssi_db;
        buffer[1] = self.rssi_percent;
        buffer[2] = self.link_quality;
        buffer[3] = self.snr as u8;
        buffer[4] = self.rf_power_db;
        buffer[5] = self.fps;
        if let (Some(rssi_ant2_db), Some(active_antenna)) = (self.rssi_ant2_db, self.active_antenna)
        {
            buffer[6] = rssi_ant2_db;
            buffer[7] = active_antenna;
        }
        Ok(payload_size)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE && data.len() != Self::EXTENDED_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Ok(Self {
            rssi_db: data[0],
            rssi_percent: data[1],
            link_quality: data[2],
            snr: data[3] as i8,
            rf_power_db: data[4],
            fps: data[5],
            rssi_ant2_db: data.get(6).copied(),
            active_antenna: data.get(7).copied(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_statistics_tx_new() {
        let packet = LinkStatisticsTx::new(100, 75, 90, -10, 20, 50).unwrap();
        assert_eq!(packet.rssi_db, 100);
        assert_eq!(packet.rssi_percent, 75);
        assert_eq!(packet.link_quality, 90);
        assert_eq!(packet.snr, -10);
        assert_eq!(packet.rf_power_db, 20);
        assert_eq!(packet.fps, 50);
    }

    #[test]
    fn test_link_statistics_tx_to_bytes() {
        let link_statistics_tx = LinkStatisticsTx {
            rssi_db: 100,
            rssi_percent: 75,
            link_quality: 90,
            snr: -10,
            rf_power_db: 20,
            fps: 50,
            rssi_ant2_db: None,
            active_antenna: None,
        };

        let mut buffer = [0u8; LinkStatisticsTx::MIN_PAYLOAD_SIZE];
        link_statistics_tx.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; LinkStatisticsTx::MIN_PAYLOAD_SIZE] = [100, 75, 90, 246, 20, 50];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_link_statistics_tx_from_bytes() {
        let data: [u8; LinkStatisticsTx::MIN_PAYLOAD_SIZE] = [100, 75, 90, 246, 20, 50];

        let link_statistics_tx = LinkStatisticsTx::from_bytes(&data).unwrap();

        assert_eq!(
            link_statistics_tx,
            LinkStatisticsTx {
                rssi_db: 100,
                rssi_percent: 75,
                link_quality: 90,
                snr: -10,
                rf_power_db: 20,
                fps: 50,
                rssi_ant2_db: None,
                active_antenna: None,
            }
        );
    }

    #[test]
    fn test_link_statistics_tx_round_trip() {
        let link_statistics_tx = LinkStatisticsTx {
            rssi_db: 100,
            rssi_percent: 75,
            link_quality: 90,
            snr: -10,
            rf_power_db: 20,
            fps: 50,
            rssi_ant2_db: None,
            active_antenna: None,
        };

        let mut buffer = [0u8; LinkStatisticsTx::MIN_PAYLOAD_SIZE];
        link_statistics_tx.to_bytes(&mut buffer).unwrap();

        let round_trip_link_statistics_tx = LinkStatisticsTx::from_bytes(&buffer).unwrap();

        assert_eq!(link_statistics_tx, round_trip_link_statistics_tx);
    }

    #[test]
    fn test_edge_cases() {
        let link_statistics_tx = LinkStatisticsTx {
            rssi_db: 255,
            rssi_percent: 100,
            link_quality: 100,
            snr: -128,
            rf_power_db: 50,
            fps: 255,
            rssi_ant2_db: None,
            active_antenna: None,
        };

        let mut buffer = [0u8; LinkStatisticsTx::MIN_PAYLOAD_SIZE];
        link_statistics_tx.to_bytes(&mut buffer).unwrap();
        let round_trip_link_statistics_tx = LinkStatisticsTx::from_bytes(&buffer).unwrap();
        assert_eq!(link_statistics_tx, round_trip_link_statistics_tx);
    }

    #[test]
    fn test_extended_wire_layout() {
        let data = [100, 75, 90, 246, 20, 50, 110, 1];
        let mut expected = LinkStatisticsTx::new(100, 75, 90, -10, 20, 50).unwrap();
        expected.rssi_ant2_db = Some(110);
        expected.active_antenna = Some(1);
        assert_eq!(LinkStatisticsTx::from_bytes(&data), Ok(expected.clone()));

        let mut buffer = [0xaa; LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE + 1];
        assert_eq!(expected.to_bytes(&mut buffer), Ok(data.len()));
        assert_eq!(&buffer[..data.len()], &data);
        assert_eq!(buffer[data.len()], 0xaa);
    }

    #[test]
    fn test_legacy_extension_unavailable() {
        let data = [100, 75, 90, 246, 20, 50];
        let packet = LinkStatisticsTx::from_bytes(&data).unwrap();
        assert_eq!(packet.rssi_ant2_db, None);
        assert_eq!(packet.active_antenna, None);
        let mut buffer = [0xaa; LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE];
        assert_eq!(packet.to_bytes(&mut buffer), Ok(data.len()));
        assert_eq!(&buffer[..data.len()], &data);
        assert_eq!(&buffer[data.len()..], &[0xaa, 0xaa]);
    }

    #[test]
    fn test_extended_zero_and_snr_limits() {
        for snr in [i8::MIN, i8::MAX] {
            let mut data = [100, 75, 90, 246, 20, 50, 110, 1];
            data[3] = snr as u8;
            data[6] = 0;
            data[7] = 0;
            let packet = LinkStatisticsTx::from_bytes(&data).unwrap();
            assert_eq!(packet.snr, snr);
            assert_eq!(packet.rssi_ant2_db, Some(0));
            assert_eq!(packet.active_antenna, Some(0));
            let mut buffer = [0; LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE];
            assert_eq!(packet.to_bytes(&mut buffer), Ok(data.len()));
            assert_eq!(buffer, data);
        }
    }

    #[test]
    fn test_invalid_payload_lengths() {
        let data = [0; LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE + 1];
        for len in 0..=data.len() {
            if len != LinkStatisticsTx::MIN_PAYLOAD_SIZE
                && len != LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE
            {
                assert_eq!(
                    LinkStatisticsTx::from_bytes(&data[..len]),
                    Err(CrsfParsingError::InvalidPayloadLength)
                );
            }
        }
    }

    #[test]
    fn test_short_buffers() {
        let mut packet = LinkStatisticsTx::new(100, 75, 90, -10, 20, 50).unwrap();
        for extended in [false, true] {
            let size = if extended {
                packet.rssi_ant2_db = Some(110);
                packet.active_antenna = Some(1);
                LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE
            } else {
                LinkStatisticsTx::MIN_PAYLOAD_SIZE
            };
            for len in 0..size {
                let mut buffer = [0xaa; LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE];
                assert_eq!(
                    packet.to_bytes(&mut buffer[..len]),
                    Err(CrsfParsingError::BufferOverflow)
                );
                assert_eq!(buffer, [0xaa; LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE]);
            }
        }
    }

    #[test]
    fn test_incomplete_extension() {
        let mut packet = LinkStatisticsTx::new(100, 75, 90, -10, 20, 50).unwrap();
        for (rssi, antenna) in [(Some(110), None), (None, Some(1))] {
            packet.rssi_ant2_db = rssi;
            packet.active_antenna = antenna;
            let mut buffer = [0xaa; LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE];
            assert_eq!(
                packet.to_bytes(&mut buffer),
                Err(CrsfParsingError::InvalidPayload)
            );
            assert_eq!(buffer, [0xaa; LinkStatisticsTx::EXTENDED_PAYLOAD_SIZE]);
        }
    }
}
