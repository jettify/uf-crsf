use heapless::String;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlightMode {
    pub flight_mode: String<63>,
}

impl FlightMode {
    pub const MAX_LEN: usize = 64;

    pub fn to_bytes(&self, buffer: &mut [u8]) -> usize {
        let bytes = self.flight_mode.as_bytes();
        let len = bytes.len();
        if len < buffer.len() {
            buffer[..len].copy_from_slice(bytes);
            buffer[len] = 0; // Null terminator
            len + 1
        } else {
            0
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let null_pos = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        let s = core::str::from_utf8(&data[..null_pos]).unwrap_or("");
        let mut flight_mode = String::new();
        flight_mode.push_str(s).unwrap();
        Self { flight_mode }
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

        let mut buffer = [0u8; FlightMode::MAX_LEN];
        let len = flight_mode.to_bytes(&mut buffer);

        let expected_bytes: [u8; 5] = [b'A', b'C', b'R', b'O', 0];

        assert_eq!(len, 5);
        assert_eq!(&buffer[..len], &expected_bytes);
    }

    #[test]
    fn test_flight_mode_from_bytes() {
        let data: [u8; 5] = [b'A', b'C', b'R', b'O', 0];
        let flight_mode = FlightMode::from_bytes(&data);

        let mut expected_flight_mode_str: String<63> = String::new();
        expected_flight_mode_str.push_str("ACRO").unwrap();
        assert_eq!(flight_mode.flight_mode, expected_flight_mode_str);
    }

    #[test]
    fn test_flight_mode_from_bytes_no_null() {
        let data: [u8; 4] = [b'A', b'C', b'R', b'O'];
        let flight_mode = FlightMode::from_bytes(&data);

        let mut expected_flight_mode_str: String<63> = String::new();
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

        let mut buffer = [0u8; FlightMode::MAX_LEN];
        let len = flight_mode.to_bytes(&mut buffer);

        let round_trip_flight_mode = FlightMode::from_bytes(&buffer[..len]);

        assert_eq!(flight_mode, round_trip_flight_mode);
    }

    #[test]
    fn test_empty_flight_mode() {
        let flight_mode_str = String::new();
        let flight_mode = FlightMode {
            flight_mode: flight_mode_str,
        };

        let mut buffer = [0u8; FlightMode::MAX_LEN];
        let len = flight_mode.to_bytes(&mut buffer);
        let round_trip_flight_mode = FlightMode::from_bytes(&buffer[..len]);
        assert_eq!(flight_mode, round_trip_flight_mode);
        assert_eq!(len, 1);
        assert_eq!(buffer[0], 0);
    }
}