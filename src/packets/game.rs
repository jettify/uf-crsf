use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;
use core::mem::size_of;

const ADD_POINTS_SUB_TYPE: u8 = 0x01;
const COMMAND_CODE_SUB_TYPE: u8 = 0x02;

/// Represents a Game packet (frame type 0x3C).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Game {
    pub dst_addr: u8,
    pub src_addr: u8,
    pub payload: GamePayload,
}

/// Enum for the different payloads of a Game packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GamePayload {
    AddPoints(i16),
    CommandCode(u16),
}

impl CrsfPacket for Game {
    const PACKET_TYPE: PacketType = PacketType::Game;
    // Dst + Src + Sub-type + max payload size (i16/u16)
    const MIN_PAYLOAD_SIZE: usize = 3 * size_of::<u8>() + size_of::<u16>();

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let dst_addr = data[0];
        let src_addr = data[1];
        let sub_type = data[2];
        let sub_payload = &data[3..5];

        let payload = match sub_type {
            ADD_POINTS_SUB_TYPE => GamePayload::AddPoints(i16::from_be_bytes(
                sub_payload[0..size_of::<i16>()]
                    .try_into()
                    .expect("infallible due to length check"),
            )),
            COMMAND_CODE_SUB_TYPE => GamePayload::CommandCode(u16::from_be_bytes(
                sub_payload[0..size_of::<u16>()]
                    .try_into()
                    .expect("infallible due to length check"),
            )),
            _ => return Err(CrsfParsingError::InvalidPayload), // Unknown sub-type
        };

        Ok(Self {
            dst_addr,
            src_addr,
            payload,
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let (sub_type, payload_bytes) = match &self.payload {
            GamePayload::AddPoints(points) => (ADD_POINTS_SUB_TYPE, points.to_be_bytes()),
            GamePayload::CommandCode(code) => (COMMAND_CODE_SUB_TYPE, code.to_be_bytes()),
        };

        let payload_len = payload_bytes.len();
        let total_len = 2 + 1 + payload_len;

        if buffer.len() < total_len {
            return Err(CrsfParsingError::BufferOverflow);
        }

        buffer[0] = self.dst_addr;
        buffer[1] = self.src_addr;
        buffer[2] = sub_type;
        buffer[3..3 + payload_len].copy_from_slice(&payload_bytes);

        Ok(total_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_points_from_bytes() {
        let data: [u8; 5] = [0xEA, 0xEE, ADD_POINTS_SUB_TYPE, 0x00, 0x64]; // 100 points
        let packet = Game::from_bytes(&data).unwrap();
        assert_eq!(packet.dst_addr, 0xEA);
        assert_eq!(packet.src_addr, 0xEE);
        assert!(matches!(packet.payload, GamePayload::AddPoints(points) if points ==  100));
    }

    #[test]
    fn test_add_points_to_bytes() {
        let packet = Game {
            dst_addr: 0xEA,
            src_addr: 0xEE,
            payload: GamePayload::AddPoints(100),
        };
        let mut buffer = [0u8; 5];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 5);
        let expected: [u8; 5] = [0xEA, 0xEE, ADD_POINTS_SUB_TYPE, 0x00, 0x64];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_command_code_from_bytes() {
        let data: [u8; 5] = [0xC8, 0xEC, COMMAND_CODE_SUB_TYPE, 0x12, 0x34]; // code 0x1234
        let packet = Game::from_bytes(&data).unwrap();
        assert_eq!(packet.dst_addr, 0xC8);
        assert_eq!(packet.src_addr, 0xEC);
        assert!(matches!(packet.payload, GamePayload::CommandCode(code) if code ==  0x1234));
    }

    #[test]
    fn test_command_code_to_bytes() {
        let packet = Game {
            dst_addr: 0xC8,
            src_addr: 0xEC,
            payload: GamePayload::CommandCode(0x1234),
        };
        let mut buffer = [0u8; 5];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 5);
        let expected: [u8; 5] = [0xC8, 0xEC, COMMAND_CODE_SUB_TYPE, 0x12, 0x34];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_game_round_trip() {
        let packet = Game {
            dst_addr: 0xC8,
            src_addr: 0xEC,
            payload: GamePayload::AddPoints(-50),
        };
        let mut buffer = [0u8; 5];
        packet.to_bytes(&mut buffer).unwrap();
        let round_trip = Game::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip);
    }

    #[test]
    fn test_game_from_bytes_too_small() {
        let data: [u8; 4] = [0; 4];
        let result = Game::from_bytes(&data);
        assert_eq!(result, Err(CrsfParsingError::InvalidPayloadLength));
    }

    #[test]
    fn test_game_to_bytes_too_small() {
        let packet = Game {
            dst_addr: 0xC8,
            src_addr: 0xEC,
            payload: GamePayload::AddPoints(-50),
        };
        let mut buffer = [0u8; 4];
        let result = packet.to_bytes(&mut buffer);
        assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
    }
}
