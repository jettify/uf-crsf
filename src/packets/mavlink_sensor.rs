use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;
use core::mem::size_of;

/// Represents a MAVLink System Status Sensor packet (frame type 0xAC).
///
/// To decode data packed within the frame, please refer to the official MAVLink documentation.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MavLinkSensor {
    pub dst_addr: u8,
    pub src_addr: u8,
    /// Bitmask of sensors present.
    pub sensor_present: u32,
    /// Bitmask of sensors enabled.
    pub sensor_enabled: u32,
    /// Bitmask of sensors health.
    pub sensor_health: u32,
}

impl MavLinkSensor {
    pub fn new(
        dst_addr: u8,
        src_addr: u8,
        sensor_present: u32,
        sensor_enabled: u32,
        sensor_health: u32,
    ) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            dst_addr,
            src_addr,
            sensor_present,
            sensor_enabled,
            sensor_health,
        })
    }
}

impl CrsfPacket for MavLinkSensor {
    const PACKET_TYPE: PacketType = PacketType::MavLinkSensor;
    const MIN_PAYLOAD_SIZE: usize = 2 * size_of::<u8>() + 3 * size_of::<u32>();

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        Ok(Self {
            dst_addr: data[0],
            src_addr: data[1],
            sensor_present: u32::from_be_bytes(data[2..6].try_into().expect("infallible")),
            sensor_enabled: u32::from_be_bytes(data[6..10].try_into().expect("infallible")),
            sensor_health: u32::from_be_bytes(data[10..14].try_into().expect("infallible")),
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0] = self.dst_addr;
        buffer[1] = self.src_addr;
        buffer[2..6].copy_from_slice(&self.sensor_present.to_be_bytes());
        buffer[6..10].copy_from_slice(&self.sensor_enabled.to_be_bytes());
        buffer[10..14].copy_from_slice(&self.sensor_health.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mavlink_sensor_from_bytes() {
        assert_eq!(MavLinkSensor::MIN_PAYLOAD_SIZE, 14);
        let data: [u8; 14] = [
            0x00, 0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
        ];
        let packet = MavLinkSensor::from_bytes(&data).unwrap();

        assert_eq!(packet.dst_addr, 0x00);
        assert_eq!(packet.src_addr, 0x01);
        assert_eq!(packet.sensor_present, 0x01020304);
        assert_eq!(packet.sensor_enabled, 0x05060708);
        assert_eq!(packet.sensor_health, 0x090A0B0C);
    }

    #[test]
    fn test_mavlink_sensor_to_bytes() {
        let packet = MavLinkSensor {
            sensor_present: 0x01020304,
            dst_addr: 0,
            src_addr: 1,
            sensor_enabled: 0x05060708,
            sensor_health: 0x090A0B0C,
        };
        let mut buffer = [0u8; 14];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 14);
        let expected: [u8; 14] = [
            0x00, 0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_mavlink_sensor_round_trip() {
        let packet = MavLinkSensor {
            dst_addr: 0,
            src_addr: 1,
            sensor_present: 123,
            sensor_enabled: 456,
            sensor_health: 789,
        };
        let mut buffer = [0u8; 14];
        packet.to_bytes(&mut buffer).unwrap();
        let round_trip = MavLinkSensor::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip);
    }

    #[test]
    fn test_mavlink_sensor_from_bytes_too_small() {
        let data: [u8; 13] = [0; 13];
        let result = MavLinkSensor::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }

    #[test]
    fn test_mavlink_sensor_to_bytes_too_small() {
        let packet = MavLinkSensor {
            dst_addr: 0,
            src_addr: 1,
            sensor_present: 1,
            sensor_enabled: 2,
            sensor_health: 3,
        };
        let mut buffer = [0u8; 13];
        let result = packet.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }
}
