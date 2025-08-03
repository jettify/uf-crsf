use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;
use heapless::String;

const MAX_FLIGHT_MODE_LEN: usize = 59;

/// Represents a Flight Mode packet.
///
/// Contains the flight mode as a null-terminated string.
#[derive(Clone, Debug, PartialEq)]
pub struct FlightMode {
    /// The flight mode string.
    pub flight_mode: String<MAX_FLIGHT_MODE_LEN>,
}

#[cfg(feature = "defmt")]
impl defmt::Format for FlightMode {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "FlightMode {{ flight_mode: {} }}",
            self.flight_mode.as_str(),
        )
    }
}

impl CrsfPacket for FlightMode {
    const PACKET_TYPE: PacketType = PacketType::FlightMode;
    // An empty flight mode is a single null byte
    const MIN_PAYLOAD_SIZE: usize = 1;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let bytes = self.flight_mode.as_bytes();
        let len_with_null = bytes.len() + 1;
        if buffer.len() < len_with_null {
            return Err(CrsfParsingError::BufferOverflow);
        }
        buffer[..bytes.len()].copy_from_slice(bytes);
        buffer[bytes.len()] = 0; // Null terminator
        Ok(len_with_null)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        let null_pos = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        let s = core::str::from_utf8(&data[..null_pos])
            .map_err(|_| CrsfParsingError::InvalidPayload)?;
        let mut flight_mode = String::new();
        flight_mode
            .push_str(s)
            .map_err(|()| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self { flight_mode })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_mode_to_bytes() {
        let mut flight_mode_str = String::new();
        flight_mode_str.push_str("ACRO").unwrap();
        let flight_mode = FlightMode {
            flight_mode: flight_mode_str,
        };

        let mut buffer = [0u8; 60];
        let len = flight_mode.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; 5] = [b'A', b'C', b'R', b'O', 0];

        assert_eq!(len, 5);
        assert_eq!(&buffer[..len], &expected_bytes);
    }

    #[test]
    fn test_flight_mode_to_bytes_buffer_too_small() {
        let mut flight_mode_str = String::new();
        flight_mode_str.push_str("LONG FLIGHT MODE").unwrap();
        let flight_mode = FlightMode {
            flight_mode: flight_mode_str,
        };

        let mut buffer = [0u8; 10];
        let result = flight_mode.to_bytes(&mut buffer);
        assert!(matches!(result, Err(CrsfParsingError::BufferOverflow)));
    }

    #[test]
    fn test_flight_mode_from_bytes() {
        let data: [u8; 5] = [b'A', b'C', b'R', b'O', 0];
        let flight_mode = FlightMode::from_bytes(&data).unwrap();

        let mut expected_flight_mode_str: String<MAX_FLIGHT_MODE_LEN> = String::new();
        expected_flight_mode_str.push_str("ACRO").unwrap();
        assert_eq!(flight_mode.flight_mode, expected_flight_mode_str);
    }

    #[test]
    fn test_flight_mode_from_bytes_no_null() {
        let data: [u8; 4] = [b'A', b'C', b'R', b'O'];
        let flight_mode = FlightMode::from_bytes(&data).unwrap();

        let mut expected_flight_mode_str: String<MAX_FLIGHT_MODE_LEN> = String::new();
        expected_flight_mode_str.push_str("ACRO").unwrap();
        assert_eq!(flight_mode.flight_mode, expected_flight_mode_str);
    }

    #[test]
    fn test_flight_mode_round_trip() {
        let mut flight_mode_str = String::new();
        flight_mode_str.push_str("STABILIZE").unwrap();
        let flight_mode = FlightMode {
            flight_mode: flight_mode_str,
        };

        let mut buffer = [0u8; 60];
        let len = flight_mode.to_bytes(&mut buffer).unwrap();

        let round_trip_flight_mode = FlightMode::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(flight_mode, round_trip_flight_mode);
    }

    #[test]
    fn test_empty_flight_mode() {
        let flight_mode_str = String::new();
        let flight_mode = FlightMode {
            flight_mode: flight_mode_str,
        };

        let mut buffer = [0u8; 60];
        let len = flight_mode.to_bytes(&mut buffer).unwrap();
        let round_trip_flight_mode = FlightMode::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(flight_mode, round_trip_flight_mode);
        assert_eq!(len, 1);
        assert_eq!(buffer[0], 0);
    }
}
