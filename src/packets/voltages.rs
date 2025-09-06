use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;
use heapless::Vec;

/// Represents a Voltages packet.
///
/// Used to transmit voltage telemetry from the craft to the transmitter. Can be used
/// to report battery cell voltages, or a group of associated voltages from a single source.
#[derive(Clone, Debug, PartialEq)]
pub struct Voltages {
    /// Source of the voltages.
    pub voltage_source_id: u8,
    /// Up to 29 voltages in millivolts (e.g., 3.850V = 3850).
    voltage_values: Vec<u16, 29>,
}

impl Voltages {
    /// Creates a new Voltages packet from a slice of voltage values.
    ///
    /// The number of voltage values must be 29 or less.
    pub fn new(voltage_source_id: u8, voltage_values: &[u16]) -> Result<Self, CrsfParsingError> {
        if voltage_values.len() > 29 {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let mut values = Vec::new();
        values
            .extend_from_slice(voltage_values)
            .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self {
            voltage_source_id,
            voltage_values: values,
        })
    }

    /// Returns the voltage values as a slice.
    pub fn voltage_values(&self) -> &[u16] {
        &self.voltage_values
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Voltages {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Voltages {{ voltage_source_id: {}, voltage_values: {} }}",
            self.voltage_source_id,
            self.voltage_values(),
        )
    }
}

impl CrsfPacket for Voltages {
    const PACKET_TYPE: PacketType = PacketType::Voltages;
    const MIN_PAYLOAD_SIZE: usize = 3;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let required_len = 1 + self.voltage_values.len() * 2;
        if buffer.len() < required_len {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[0] = self.voltage_source_id;
        let mut i = 1;
        for &voltage in self.voltage_values() {
            let bytes = voltage.to_be_bytes();
            buffer[i..i + 2].copy_from_slice(&bytes);
            i += 2;
        }
        Ok(i)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let voltage_source_id = data[0];
        let voltage_values: Vec<u16, 29> = data[1..]
            .chunks_exact(2)
            .map(|chunk| {
                let bytes = [chunk[0], chunk[1]];
                u16::from_be_bytes(bytes)
            })
            .collect();

        Ok(Self {
            voltage_source_id,
            voltage_values,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voltages_to_bytes() {
        let voltage_values = [3850, 3900];
        let voltages = Voltages::new(0, &voltage_values).unwrap();

        let mut buffer = [0u8; 60];
        let len = voltages.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; 5] = [
            0, // Source ID
            0x0f, 0x0a, // 3850
            0x0f, 0x3c, // 3900
        ];

        assert_eq!(len, 5);
        assert_eq!(&buffer[..len], &expected_bytes);
    }

    #[test]
    fn test_voltages_from_bytes() {
        let data: [u8; 5] = [
            0, // Source ID
            0x0f, 0x0a, // 3850
            0x0f, 0x3c, // 3900
        ];

        let voltages = Voltages::from_bytes(&data).unwrap();

        let expected_voltage_values = [3850, 3900];
        assert_eq!(voltages.voltage_source_id, 0);
        assert_eq!(voltages.voltage_values(), &expected_voltage_values);
    }

    #[test]
    fn test_voltages_round_trip() {
        let voltage_values = [3700, 3800];
        let voltages = Voltages::new(1, &voltage_values).unwrap();

        let mut buffer = [0u8; 60];
        let len = voltages.to_bytes(&mut buffer).unwrap();

        let round_trip_voltages = Voltages::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(voltages, round_trip_voltages);
    }

    #[test]
    fn test_edge_cases() {
        let voltage_values = [0, 65535];
        let voltages = Voltages::new(2, &voltage_values).unwrap();

        let mut buffer = [0u8; 29 * 2 + 1];
        let len = voltages.to_bytes(&mut buffer).unwrap();
        let round_trip_voltages = Voltages::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(voltages, round_trip_voltages);
    }
}
