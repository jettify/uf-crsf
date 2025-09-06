use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;
use heapless::Vec;

/// Represents a Temperature packet.
///
/// Used to transmit temperature telemetry data from the vehicle to the transmitter.
/// This frame can be used to report temperature readings from various sources on the vehicle,
/// such as motors, ESCs, or the environment.
#[derive(Clone, Debug, PartialEq)]
pub struct Temp {
    /// Identifies the source of the temperature data (e.g., 0 = FC, 1 = Ambient, etc.).
    pub temp_source_id: u8,
    /// Up to 20 temperature values in deci-degrees Celsius (e.g., 250 = 25.0°C).
    temperatures: Vec<i16, 20>,
}

impl Temp {
    /// Creates a new Temp packet from a slice of temperature values.
    ///
    /// The number of temperature values must be 20 or less.
    pub fn new(temp_source_id: u8, temperatures: &[i16]) -> Result<Self, CrsfParsingError> {
        if temperatures.len() > 20 {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let mut temps = Vec::new();
        temps
            .extend_from_slice(temperatures)
            .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self {
            temp_source_id,
            temperatures: temps,
        })
    }

    /// Returns the temperature values as a slice.
    pub fn temperatures(&self) -> &[i16] {
        &self.temperatures
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Temp {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Temp {{ temp_source_id: {}, temperatures: {} }}",
            self.temp_source_id,
            self.temperatures(),
        )
    }
}

impl CrsfPacket for Temp {
    const PACKET_TYPE: PacketType = PacketType::Temp;
    const MIN_PAYLOAD_SIZE: usize = 3;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let required_len = 1 + self.temperatures.len() * 2;
        if buffer.len() < required_len {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[0] = self.temp_source_id;
        let mut i = 1;
        for &temp in self.temperatures() {
            let bytes = temp.to_be_bytes();
            buffer[i..i + 2].copy_from_slice(&bytes);
            i += 2;
        }
        Ok(i)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let temp_source_id = data[0];
        let temperatures: Vec<i16, 20> = data[1..]
            .chunks_exact(2)
            .map(|chunk| {
                let bytes = [chunk[0], chunk[1]];
                i16::from_be_bytes(bytes)
            })
            .collect();

        Ok(Self {
            temp_source_id,
            temperatures,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_to_bytes() {
        let temperatures = [250, -50];
        let temp = Temp::new(1, &temperatures).unwrap();

        let mut buffer = [0u8; 60];
        let len = temp.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; 5] = [
            1, // Source ID
            0x00, 0xfa, // 250
            0xff, 0xce, // -50
        ];

        assert_eq!(len, 5);
        assert_eq!(&buffer[..len], &expected_bytes);
    }

    #[test]
    fn test_temp_from_bytes() {
        let data: [u8; 5] = [
            1, // Source ID
            0x00, 0xfa, // 250
            0xff, 0xce, // -50
        ];

        let temp = Temp::from_bytes(&data).unwrap();

        let expected_temperatures = [250, -50];
        assert_eq!(temp.temp_source_id, 1);
        assert_eq!(temp.temperatures(), &expected_temperatures);
    }

    #[test]
    fn test_temp_round_trip() {
        let temperatures = [1234, -5678];
        let temp = Temp::new(2, &temperatures).unwrap();

        let mut buffer = [0u8; 60];
        let len = temp.to_bytes(&mut buffer).unwrap();

        let round_trip_temp = Temp::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(temp, round_trip_temp);
    }

    #[test]
    fn test_edge_cases() {
        let temperatures = [0, 32767, -32768];
        let temp = Temp::new(3, &temperatures).unwrap();

        let mut buffer = [0u8; 60];
        let len = temp.to_bytes(&mut buffer).unwrap();
        let round_trip_temp = Temp::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(temp, round_trip_temp);
    }
}
