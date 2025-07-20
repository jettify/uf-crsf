use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents a VTX Telemetry packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VtxTelemetry {
    /// Origin device address.
    pub origin_address: u8,
    /// VTX power in dBm.
    pub power_dbm: u8,
    /// VTX frequency in MHz.
    pub frequency_mhz: u16,
    /// Pit mode status (0=Off, 1=On).
    pub pit_mode: bool,
    /// Pit mode control (0=Off, 1=On, 2=Switch, 3=Failsafe).
    pub pitmode_control: u8,
    /// Pit mode switch (0=Ch5, 1=Ch5 Inv, ...).
    pub pitmode_switch: u8,
}

impl CrsfPacket for VtxTelemetry {
    const PACKET_TYPE: PacketType = PacketType::VtxTelemetry;
    const MIN_PAYLOAD_SIZE: usize = 5;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0] = self.origin_address;
        buffer[1] = self.power_dbm;
        buffer[2..4].copy_from_slice(&self.frequency_mhz.to_be_bytes());
        let pit_byte =
            (u8::from(self.pit_mode)) | (self.pitmode_control << 1) | (self.pitmode_switch << 3);
        buffer[4] = pit_byte;
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let pit_byte = data[4];
        Ok(Self {
            origin_address: data[0],
            power_dbm: data[1],
            frequency_mhz: u16::from_be_bytes(
                data[2..4]
                    .try_into()
                    .map_err(|_| CrsfParsingError::InvalidPayloadLength)?,
            ),
            pit_mode: (pit_byte & 0b1) != 0,
            pitmode_control: (pit_byte >> 1) & 0b11,
            pitmode_switch: (pit_byte >> 3) & 0b1111,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vtx_telemetry_to_bytes() {
        let vtx_telemetry = VtxTelemetry {
            origin_address: 0xEE,
            power_dbm: 25,
            frequency_mhz: 5800,
            pit_mode: true,
            pitmode_control: 2,
            pitmode_switch: 5,
        };

        let mut buffer = [0u8; VtxTelemetry::MIN_PAYLOAD_SIZE];
        vtx_telemetry.to_bytes(&mut buffer).unwrap();

        // pit_byte = 1 | (2 << 1) | (5 << 3) = 1 | 4 | 40 = 45 = 0b00101101
        let expected_bytes: [u8; VtxTelemetry::MIN_PAYLOAD_SIZE] =
            [0xEE, 25, 0x16, 0xA8, 0b00101101];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_vtx_telemetry_from_bytes() {
        let data: [u8; VtxTelemetry::MIN_PAYLOAD_SIZE] = [0xEE, 25, 0x16, 0xA8, 0b00101101];

        let vtx_telemetry = VtxTelemetry::from_bytes(&data).unwrap();

        assert_eq!(
            vtx_telemetry,
            VtxTelemetry {
                origin_address: 0xEE,
                power_dbm: 25,
                frequency_mhz: 5800,
                pit_mode: true,
                pitmode_control: 2,
                pitmode_switch: 5,
            }
        );
    }

    #[test]
    fn test_vtx_telemetry_round_trip() {
        let vtx_telemetry = VtxTelemetry {
            origin_address: 0xCE,
            power_dbm: 10,
            frequency_mhz: 5740,
            pit_mode: false,
            pitmode_control: 1,
            pitmode_switch: 10,
        };

        let mut buffer = [0u8; VtxTelemetry::MIN_PAYLOAD_SIZE];
        vtx_telemetry.to_bytes(&mut buffer).unwrap();

        let round_trip_vtx_telemetry = VtxTelemetry::from_bytes(&buffer).unwrap();

        assert_eq!(vtx_telemetry, round_trip_vtx_telemetry);
    }
}
