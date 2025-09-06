use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;
use heapless::Vec;

/// Represents an RPM packet.
///
/// Frame type used to transmit RPM (revolutions per minute) telemetry data from the craft
/// to the transmitter. This frame can be used to report motor or propeller RPM for
/// monitoring performance or diagnostics.
#[derive(Clone, Debug, PartialEq)]
pub struct Rpm {
    /// Identifies the source of the RPM data (e.g., 0 = Motor 1, 1 = Motor 2, etc.).
    pub rpm_source_id: u8,
    /// 1 to 19 RPM values, with negative ones representing the motor spinning in reverse.
    /// These are 24-bit values.
    rpm_values: Vec<i32, 19>,
}

impl Rpm {
    /// Creates a new RPM packet from a slice of RPM values.
    ///
    /// The number of RPM values must be between 1 and 19.
    pub fn new(rpm_source_id: u8, rpm_values: &[i32]) -> Result<Self, CrsfParsingError> {
        if rpm_values.is_empty() || rpm_values.len() > 19 {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let mut values = Vec::new();
        for &rpm in rpm_values {
            values
                .push(rpm)
                .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        }
        Ok(Self {
            rpm_source_id,
            rpm_values: values,
        })
    }

    /// Returns the RPM values as a slice.
    pub fn rpm_values(&self) -> &[i32] {
        &self.rpm_values
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Rpm {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Rpm {{ rpm_source_id: {}, rpm_values: {} }}",
            self.rpm_source_id,
            self.rpm_values(),
        )
    }
}

impl CrsfPacket for Rpm {
    const PACKET_TYPE: PacketType = PacketType::Rpm;
    const MIN_PAYLOAD_SIZE: usize = 3;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        buffer[0] = self.rpm_source_id;
        let mut i = 1;
        for &rpm in self.rpm_values() {
            let bytes = rpm.to_be_bytes();
            buffer[i..i + 3].copy_from_slice(&bytes[1..4]);
            i += 3;
        }
        Ok(i)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        let rpm_source_id = data[0];
        let rpm_values: Vec<i32, 19> = data[1..]
            .chunks_exact(3)
            .map(|chunk| {
                let mut bytes = [0; 4];
                bytes[1..4].copy_from_slice(chunk);
                let rpm = i32::from_be_bytes(bytes);
                // Sign extend the 24-bit value
                (rpm << 8) >> 8
            })
            .collect();

        Ok(Self {
            rpm_source_id,
            rpm_values,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpm_to_bytes() {
        let rpm_values = [1000, -2000];
        let rpm = Rpm::new(1, &rpm_values).unwrap();

        let mut buffer = [0u8; 60];
        let len = rpm.to_bytes(&mut buffer).unwrap();

        let expected_bytes: [u8; 7] = [
            1, // Source ID
            0x00, 0x03, 0xe8, // 1000
            0xff, 0xf8, 0x30, // -2000
        ];

        assert_eq!(len, 7);
        assert_eq!(&buffer[..len], &expected_bytes);
    }

    #[test]
    fn test_rpm_from_bytes() {
        let data: [u8; 7] = [
            1, // Source ID
            0x00, 0x03, 0xe8, // 1000
            0xff, 0xf8, 0x30, // -2000
        ];

        let rpm = Rpm::from_bytes(&data).unwrap();

        let expected_rpm_values = [1000, -2000];
        assert_eq!(rpm.rpm_source_id, 1);
        assert_eq!(rpm.rpm_values(), &expected_rpm_values);
    }

    #[test]
    fn test_rpm_round_trip() {
        let rpm_values = [123456, -654321];
        let rpm = Rpm::new(2, &rpm_values).unwrap();

        let mut buffer = [0u8; 60];
        let len = rpm.to_bytes(&mut buffer).unwrap();

        let round_trip_rpm = Rpm::from_bytes(&buffer[..len]).unwrap();

        assert_eq!(rpm, round_trip_rpm);
    }

    #[test]
    fn test_edge_cases() {
        let rpm_values = [0, 8388607, -8388608];
        let rpm = Rpm::new(3, &rpm_values).unwrap();

        let mut buffer = [0u8; 60];
        let len = rpm.to_bytes(&mut buffer).unwrap();
        let round_trip_rpm = Rpm::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(rpm, round_trip_rpm);
    }
}
