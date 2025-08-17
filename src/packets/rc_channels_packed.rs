use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

/// Represents an RC Channels Packed packet.
///
/// This packet contains 16 channels of RC data, each packed as an 11-bit value.
/// The values can be converted to microseconds using the formula: `(x - 992) * 5 / 8 + 1500`.
/// A center value of 1500µs corresponds to a raw value of 992.
///
/// In case of a failsafe, this frame will no longer be sent. It is recommended to
/// wait for 1 second before starting the FC failsafe routine.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RcChannelsPacked(pub [u16; 16]);

impl CrsfPacket for RcChannelsPacked {
    const PACKET_TYPE: PacketType = PacketType::RcChannelsPacked;
    const MIN_PAYLOAD_SIZE: usize = 16 * 11 / 8; // 16 channels, 11 bit each

    #[allow(clippy::cast_possible_truncation)]
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        let ch = &self.0;
        buffer[0] = (ch[0]) as u8;
        buffer[1] = ((ch[0] >> 8) | (ch[1] << 3)) as u8;
        buffer[2] = ((ch[1] >> 5) | (ch[2] << 6)) as u8;
        buffer[3] = (ch[2] >> 2) as u8;
        buffer[4] = ((ch[2] >> 10) | (ch[3] << 1)) as u8;
        buffer[5] = ((ch[3] >> 7) | (ch[4] << 4)) as u8;
        buffer[6] = ((ch[4] >> 4) | (ch[5] << 7)) as u8;
        buffer[7] = (ch[5] >> 1) as u8;
        buffer[8] = ((ch[5] >> 9) | (ch[6] << 2)) as u8;
        buffer[9] = ((ch[6] >> 6) | (ch[7] << 5)) as u8;
        buffer[10] = (ch[7] >> 3) as u8;
        buffer[11] = ch[8] as u8;
        buffer[12] = ((ch[8] >> 8) | (ch[9] << 3)) as u8;
        buffer[13] = ((ch[9] >> 5) | (ch[10] << 6)) as u8;
        buffer[14] = (ch[10] >> 2) as u8;
        buffer[15] = ((ch[10] >> 10) | (ch[11] << 1)) as u8;
        buffer[16] = ((ch[11] >> 7) | (ch[12] << 4)) as u8;
        buffer[17] = ((ch[12] >> 4) | (ch[13] << 7)) as u8;
        buffer[18] = (ch[13] >> 1) as u8;
        buffer[19] = ((ch[13] >> 9) | (ch[14] << 2)) as u8;
        buffer[20] = ((ch[14] >> 6) | (ch[15] << 5)) as u8;
        buffer[21] = (ch[15] >> 3) as u8;
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        const MASK_11BIT: u16 = 0x07FF;
        let data_u16: [u16; Self::MIN_PAYLOAD_SIZE] = core::array::from_fn(|i| u16::from(data[i]));
        let mut ch = [MASK_11BIT; 16];
        ch[0] &= data_u16[0] | (data_u16[1] << 8);
        ch[1] &= (data_u16[1] >> 3) | (data_u16[2] << 5);
        ch[2] &= (data_u16[2] >> 6) | (data_u16[3] << 2) | (data_u16[4] << 10);
        ch[3] &= (data_u16[4] >> 1) | (data_u16[5] << 7);
        ch[4] &= (data_u16[5] >> 4) | (data_u16[6] << 4);
        ch[5] &= (data_u16[6] >> 7) | (data_u16[7] << 1) | (data_u16[8] << 9);
        ch[6] &= (data_u16[8] >> 2) | (data_u16[9] << 6);
        ch[7] &= (data_u16[9] >> 5) | (data_u16[10] << 3);
        ch[8] &= data_u16[11] | (data_u16[12] << 8);
        ch[9] &= (data_u16[12] >> 3) | (data_u16[13] << 5);
        ch[10] &= (data_u16[13] >> 6) | (data_u16[14] << 2) | (data_u16[15] << 10);
        ch[11] &= (data_u16[15] >> 1) | (data_u16[16] << 7);
        ch[12] &= (data_u16[16] >> 4) | (data_u16[17] << 4);
        ch[13] &= (data_u16[17] >> 7) | (data_u16[18] << 1) | (data_u16[19] << 9);
        ch[14] &= (data_u16[19] >> 2) | (data_u16[20] << 6);
        ch[15] &= (data_u16[20] >> 5) | (data_u16[21] << 3);
        Ok(RcChannelsPacked(ch))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::{write_packet_to_buffer, PacketAddress};

    #[test]
    fn test_rc_channels_from_hardware_bytes() {
        // This is the existing test, renamed
        let payload: [u8; 22] = [
            0x03, 0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01, 0x0B, 0xF8,
            0xC0, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 252,
        ];
        let rc = RcChannelsPacked::from_bytes(&payload).unwrap();
        let mut buffer: [u8; 22] = [0; 22];
        let consumed = rc.to_bytes(&mut buffer).unwrap();
        assert_eq!(consumed, 22);
        assert_eq!(&buffer, &payload);
    }

    #[test]
    fn test_rc_channels_packed_round_trip() {
        let channels = RcChannelsPacked([
            1000, 1001, 1002, 1003, 1500, 1501, 1502, 1503, 2000, 2001, 2002, 2003, 992, 100, 500,
            1900,
        ]);

        let mut buffer: [u8; 22] = [0; 22];
        channels.to_bytes(&mut buffer).unwrap();

        let parsed_channels = RcChannelsPacked::from_bytes(&buffer).unwrap();
        assert_eq!(channels, parsed_channels);
    }

    #[test]
    fn test_from_bytes_invalid_len() {
        let raw_bytes: [u8; 21] = [0; 21];
        let result = RcChannelsPacked::from_bytes(&raw_bytes);
        assert!(matches!(
            result,
            Err(CrsfParsingError::InvalidPayloadLength)
        ));
    }

    #[test]
    fn test_to_bytes_buffer_too_small() {
        let channels = RcChannelsPacked([0; 16]);
        let mut buffer: [u8; 21] = [0; 21];
        let result = channels.to_bytes(&mut buffer);
        assert!(matches!(result, Err(CrsfParsingError::BufferOverflow)));
    }

    #[test]
    fn test_rc_channels_from_bytes() {
        assert_eq!(RcChannelsPacked::MIN_PAYLOAD_SIZE, 22);
        let channels = RcChannelsPacked([
            1000, 1001, 1002, 1003, 1500, 1501, 1502, 1503, 2000, 2001, 2002, 2003, 992, 100, 500,
            1900,
        ]);
        let mut buffer = [0u8; 64];
        let len = write_packet_to_buffer(&mut buffer, PacketAddress::Broadcast, &channels).unwrap();
        let payload = &buffer[3..len - 1];
        let parsed_channels = RcChannelsPacked::from_bytes(payload).unwrap();
        assert_eq!(channels, parsed_channels);
    }

    #[test]
    fn test_rc_channels_to_bytes() {
        let channels = RcChannelsPacked([
            1000, 1001, 1002, 1003, 1500, 1501, 1502, 1503, 2000, 2001, 2002, 2003, 992, 100, 500,
            1900,
        ]);

        let mut buffer = [0u8; 22];
        let len = channels.to_bytes(&mut buffer).unwrap();

        let mut expected_buffer = [0u8; 64];
        let expected_len =
            write_packet_to_buffer(&mut expected_buffer, PacketAddress::Broadcast, &channels)
                .unwrap();
        let expected_payload = &expected_buffer[3..expected_len - 1];

        assert_eq!(len, 22);
        assert_eq!(buffer, expected_payload);
    }
}
