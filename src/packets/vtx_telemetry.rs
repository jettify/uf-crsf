//!
//!  uint8_t     up_rssi_ant1;       // Uplink RSSI Antenna 1 (dBm * -1)
//!  uint8_t     up_rssi_ant2;       // Uplink RSSI Antenna 2 (dBm * -1)
//!  uint8_t     up_link_quality;    // Uplink Package success rate / Link quality (%)
//!  int8_t      up_snr;             // Uplink SNR (dB)
//!  uint8_t     active_antenna;     // number of currently best antenna
//!  uint8_t     rf_profile;         // enum {4fps = 0 , 50fps, 150fps}
//!  uint8_t     up_rf_power;        // enum {0mW = 0, 10mW, 25mW, 100mW,
//!                                  // 500mW, 1000mW, 2000mW, 250mW, 50mW}
//!  uint8_t     down_rssi;          // Downlink RSSI (dBm * -1)
//!  uint8_t     down_link_quality;  // Downlink Package success rate / Link quality (%)
//!  int8_t      down_snr;           // Downlink SNR (dB)
use crate::CrsfParsingError;
use crate::packets::CrsfPacket;
use crate::packets::PacketType;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VtxTelemetry {
    pub up_rssi_ant1: u8,
    pub up_rssi_ant2: u8,
    pub up_link_quality: u8,
    pub up_snr: i8,
    pub active_antenna: u8,
    pub rf_profile: u8,
    pub up_rf_power: u8,
    pub down_rssi: u8,
    pub down_link_quality: u8,
    pub down_snr: i8,
}

impl CrsfPacket for VtxTelemetry {
    const PACKET_TYPE: PacketType = PacketType::VtxTelemetry;
    const MIN_PAYLOAD_SIZE: usize = 10;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0] = self.up_rssi_ant1;
        buffer[1] = self.up_rssi_ant2;
        buffer[2] = self.up_link_quality;
        buffer[3] = self.up_snr as u8;
        buffer[4] = self.active_antenna;
        buffer[5] = self.rf_profile;
        buffer[6] = self.up_rf_power;
        buffer[7] = self.down_rssi;
        buffer[8] = self.down_link_quality;
        buffer[9] = self.down_snr as u8;
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            Err(CrsfParsingError::InvalidPayloadLength)
        } else {
            Ok(Self {
                up_rssi_ant1: data[0],
                up_rssi_ant2: data[1],
                up_link_quality: data[2],
                up_snr: data[3] as i8,
                active_antenna: data[4],
                rf_profile: data[5],
                up_rf_power: data[6],
                down_rssi: data[7],
                down_link_quality: data[8],
                down_snr: data[9] as i8,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vtx_telemetry_to_bytes() {
        let vtx_telemetry = VtxTelemetry {
            up_rssi_ant1: 100,
            up_rssi_ant2: 101,
            up_link_quality: 102,
            up_snr: -20,
            active_antenna: 1,
            rf_profile: 2,
            up_rf_power: 3,
            down_rssi: 4,
            down_link_quality: 5,
            down_snr: -6,
        };

        let mut buffer = [0u8; VtxTelemetry::MIN_PAYLOAD_SIZE];
        vtx_telemetry.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; VtxTelemetry::MIN_PAYLOAD_SIZE] =
            [100, 101, 102, 236, 1, 2, 3, 4, 5, 250];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_vtx_telemetry_from_bytes() {
        let data: [u8; VtxTelemetry::MIN_PAYLOAD_SIZE] = [100, 101, 102, 236, 1, 2, 3, 4, 5, 250];

        let vtx_telemetry = VtxTelemetry::from_bytes(&data).unwrap();

        assert_eq!(
            vtx_telemetry,
            VtxTelemetry {
                up_rssi_ant1: 100,
                up_rssi_ant2: 101,
                up_link_quality: 102,
                up_snr: -20,
                active_antenna: 1,
                rf_profile: 2,
                up_rf_power: 3,
                down_rssi: 4,
                down_link_quality: 5,
                down_snr: -6,
            }
        );
    }

    #[test]
    fn test_vtx_telemetry_round_trip() {
        let vtx_telemetry = VtxTelemetry {
            up_rssi_ant1: 100,
            up_rssi_ant2: 101,
            up_link_quality: 102,
            up_snr: -20,
            active_antenna: 1,
            rf_profile: 2,
            up_rf_power: 3,
            down_rssi: 4,
            down_link_quality: 5,
            down_snr: -6,
        };

        let mut buffer = [0u8; VtxTelemetry::MIN_PAYLOAD_SIZE];
        vtx_telemetry.to_bytes(&mut buffer).unwrap();

        let round_trip_vtx_telemetry = VtxTelemetry::from_bytes(&buffer).unwrap();

        assert_eq!(vtx_telemetry, round_trip_vtx_telemetry);
    }

    #[test]
    fn test_edge_cases() {
        let vtx_telemetry = VtxTelemetry {
            up_rssi_ant1: 255,
            up_rssi_ant2: 0,
            up_link_quality: 100,
            up_snr: 127,
            active_antenna: 255,
            rf_profile: 0,
            up_rf_power: 255,
            down_rssi: 0,
            down_link_quality: 255,
            down_snr: -128,
        };

        let mut buffer = [0u8; VtxTelemetry::MIN_PAYLOAD_SIZE];
        vtx_telemetry.to_bytes(&mut buffer).unwrap();
        let round_trip_vtx_telemetry = VtxTelemetry::from_bytes(&buffer).unwrap();
        assert_eq!(vtx_telemetry, round_trip_vtx_telemetry);
    }
}
