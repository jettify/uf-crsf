use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;

/// Represents an Accel/Gyro packet (frame type `0x13`).
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AccelGyro {
    /// Timestamp of the sample in microseconds.
    pub sample_time: u32,
    /// Gyro X in `INT16_MAX / 2000 DPS` units.
    pub gyro_x: i16,
    /// Gyro Y in `INT16_MAX / 2000 DPS` units.
    pub gyro_y: i16,
    /// Gyro Z in `INT16_MAX / 2000 DPS` units.
    pub gyro_z: i16,
    /// Accel X in `INT16_MAX / 16G` units.
    pub acc_x: i16,
    /// Accel Y in `INT16_MAX / 16G` units.
    pub acc_y: i16,
    /// Accel Z in `INT16_MAX / 16G` units.
    pub acc_z: i16,
    /// Gyro temperature in centidegrees Celsius.
    pub gyro_temp: i16,
}

impl AccelGyro {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sample_time: u32,
        gyro_x: i16,
        gyro_y: i16,
        gyro_z: i16,
        acc_x: i16,
        acc_y: i16,
        acc_z: i16,
        gyro_temp: i16,
    ) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            sample_time,
            gyro_x,
            gyro_y,
            gyro_z,
            acc_x,
            acc_y,
            acc_z,
            gyro_temp,
        })
    }
}

impl CrsfPacket for AccelGyro {
    const PACKET_TYPE: PacketType = PacketType::AccelGyro;
    const MIN_PAYLOAD_SIZE: usize = 18;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            sample_time: u32::from_be_bytes(data[0..4].try_into().expect("infallible")),
            gyro_x: i16::from_be_bytes(data[4..6].try_into().expect("infallible")),
            gyro_y: i16::from_be_bytes(data[6..8].try_into().expect("infallible")),
            gyro_z: i16::from_be_bytes(data[8..10].try_into().expect("infallible")),
            acc_x: i16::from_be_bytes(data[10..12].try_into().expect("infallible")),
            acc_y: i16::from_be_bytes(data[12..14].try_into().expect("infallible")),
            acc_z: i16::from_be_bytes(data[14..16].try_into().expect("infallible")),
            gyro_temp: i16::from_be_bytes(data[16..18].try_into().expect("infallible")),
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[0..4].copy_from_slice(&self.sample_time.to_be_bytes());
        buffer[4..6].copy_from_slice(&self.gyro_x.to_be_bytes());
        buffer[6..8].copy_from_slice(&self.gyro_y.to_be_bytes());
        buffer[8..10].copy_from_slice(&self.gyro_z.to_be_bytes());
        buffer[10..12].copy_from_slice(&self.acc_x.to_be_bytes());
        buffer[12..14].copy_from_slice(&self.acc_y.to_be_bytes());
        buffer[14..16].copy_from_slice(&self.acc_z.to_be_bytes());
        buffer[16..18].copy_from_slice(&self.gyro_temp.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accel_gyro_new() {
        let packet = AccelGyro::new(1, 2, 3, 4, 5, 6, 7, 8).unwrap();
        assert_eq!(packet.sample_time, 1);
        assert_eq!(packet.gyro_x, 2);
        assert_eq!(packet.gyro_y, 3);
        assert_eq!(packet.gyro_z, 4);
        assert_eq!(packet.acc_x, 5);
        assert_eq!(packet.acc_y, 6);
        assert_eq!(packet.acc_z, 7);
        assert_eq!(packet.gyro_temp, 8);
    }

    #[test]
    fn test_accel_gyro_to_bytes() {
        let packet = AccelGyro {
            sample_time: 0x11223344,
            gyro_x: 0x0102,
            gyro_y: -2,
            gyro_z: 0x0304,
            acc_x: -3,
            acc_y: 0x0506,
            acc_z: -4,
            gyro_temp: 2500,
        };
        let mut buffer = [0u8; AccelGyro::MIN_PAYLOAD_SIZE];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, AccelGyro::MIN_PAYLOAD_SIZE);
        assert_eq!(
            buffer,
            [
                0x11, 0x22, 0x33, 0x44, 0x01, 0x02, 0xFF, 0xFE, 0x03, 0x04, 0xFF, 0xFD, 0x05, 0x06,
                0xFF, 0xFC, 0x09, 0xC4
            ]
        );
    }

    #[test]
    fn test_accel_gyro_from_bytes() {
        let data = [
            0x00, 0x00, 0x00, 0x64, 0x03, 0xE8, 0xFE, 0x0C, 0x00, 0x7B, 0x01, 0xF4, 0xFE, 0xD4,
            0xFF, 0x9C, 0x00, 0xFA,
        ];
        let packet = AccelGyro::from_bytes(&data).unwrap();
        assert_eq!(packet.sample_time, 100);
        assert_eq!(packet.gyro_x, 1_000);
        assert_eq!(packet.gyro_y, -500);
        assert_eq!(packet.gyro_z, 123);
        assert_eq!(packet.acc_x, 500);
        assert_eq!(packet.acc_y, -300);
        assert_eq!(packet.acc_z, -100);
        assert_eq!(packet.gyro_temp, 250);
    }

    #[test]
    fn test_accel_gyro_round_trip() {
        let packet = AccelGyro {
            sample_time: u32::MAX,
            gyro_x: i16::MIN,
            gyro_y: i16::MAX,
            gyro_z: 0,
            acc_x: 42,
            acc_y: -42,
            acc_z: 1,
            gyro_temp: -327,
        };
        let mut buffer = [0u8; AccelGyro::MIN_PAYLOAD_SIZE];
        packet.to_bytes(&mut buffer).unwrap();
        let round_trip = AccelGyro::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip);
    }

    #[test]
    fn test_accel_gyro_to_bytes_too_small() {
        let packet = AccelGyro::new(1, 1, 1, 1, 1, 1, 1, 1).unwrap();
        let mut buffer = [0u8; 17];
        let result = packet.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }

    #[test]
    fn test_accel_gyro_from_bytes_invalid_length() {
        let data = [0u8; 17];
        let result = AccelGyro::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }
}
