#![no_std]
// use num_enum::TryFromPrimitive;
mod packets;
use packets::CrsfParsingError;
use packets::Packet;
use packets::RawCrsfPacket;
extern crate std;

pub mod constants {
    pub const CRSF_SYNC_BYTE: u8 = 0xC8;
    pub const CRSF_MAX_PACKET_SIZE: usize = 64;
    // Header (1) + Type (1) + CRC (1) + Payload (min 1)
    pub const CRSF_MIN_PACKET_SIZE: usize = 4;
    pub const CRSF_PACKET_HEADER_LEN: usize = 2;
}

#[derive(Debug, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum State {
    #[default]
    AwaitingHead,
    AwaitingLenth,
    Reading(usize),
}

#[derive(Debug)]
pub struct CrsfParser {
    buffer: [u8; constants::CRSF_MAX_PACKET_SIZE],
    state: State,
    position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrsfError {
    InvalidPacketLength,
    InvalidCrc { calculated_crc: u8, packet_crc: u8 },
    NeedMoreData(usize),
    UnexpectedPacketType(u8),
    ParsingError(CrsfParsingError),
}

impl CrsfParser {
    pub fn new() -> Self {
        Self {
            buffer: [0; constants::CRSF_MAX_PACKET_SIZE],
            state: State::AwaitingHead,
            position: 0,
        }
    }

    pub fn push_byte_raw(&mut self, byte: u8) -> Result<RawCrsfPacket<'_>, CrsfError> {
        match self.state {
            State::AwaitingHead => {
                self.position = 0;
                self.buffer[self.position] = byte;
                self.state = State::AwaitingLenth;
                Err(CrsfError::NeedMoreData(1))
            }
            State::AwaitingLenth => {
                let n = byte as usize + 2;

                if !(constants::CRSF_MIN_PACKET_SIZE..constants::CRSF_MAX_PACKET_SIZE).contains(&n)
                {
                    self.reset();
                    return Err(CrsfError::InvalidPacketLength);
                }
                self.position = 1;
                self.buffer[self.position] = byte;
                self.state = State::Reading(n);
                Err(CrsfError::NeedMoreData(n))
            }
            State::Reading(n) if self.position == n - 2 => {
                self.position += 1;
                self.buffer[self.position] = byte;

                let crc8_dvb_s2 = crc::Crc::<u8>::new(&crc::CRC_8_DVB_S2);
                let mut digest = crc8_dvb_s2.digest();
                digest.update(&self.buffer[2..self.position]);
                let calculated_crc = digest.finalize();
                let packet_crc = self.buffer[self.position];

                if calculated_crc != packet_crc {
                    self.reset();
                    return Err(CrsfError::InvalidCrc {
                        calculated_crc,
                        packet_crc,
                    });
                }
                let start = 0;
                let end = self.position + 1;
                self.reset();
                let bytes = &self.buffer[start..end];
                Ok(RawCrsfPacket::new(bytes).unwrap())
            }
            State::Reading(n) => {
                self.position += 1;
                self.buffer[self.position] = byte;
                Err(CrsfError::NeedMoreData(n - self.position))
            }
        }
    }

    pub fn iter_packets<'a, 'b>(&'a mut self, buffer: &'b [u8]) -> PacketIterator<'a, 'b> {
        PacketIterator {
            parser: self,
            buffer,
        }
    }

    pub fn push_byte(&mut self, byte: u8) -> Result<Packet, CrsfError> {
        let raw_packet = self.push_byte_raw(byte)?;
        Packet::parse(&raw_packet).map_err(CrsfError::ParsingError)
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.state = State::AwaitingHead
    }
}

pub struct PacketIterator<'a, 'b> {
    parser: &'a mut CrsfParser,
    buffer: &'b [u8],
}

impl Iterator for PacketIterator<'_, '_> {
    type Item = Result<Packet, CrsfError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.buffer.is_empty() {
                break;
            }

            let byte = self.buffer[0];
            self.buffer = &self.buffer[1..];

            match self.parser.push_byte(byte) {
                Ok(result) => return Some(Ok(result)),
                Err(CrsfError::NeedMoreData(_)) => (),
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}

impl Default for CrsfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::packets::PacketType;
    use crate::packets::{LinkStatistics, Packet, RcChannelsPacked};

    use super::*;

    #[test]
    fn test_construction() {
        let raw_bytes: [u8; 14] = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
        let mut parser = CrsfParser::new();

        for b in &raw_bytes[0..raw_bytes.len() - 1] {
            assert!(parser.push_byte_raw(*b).is_err());
        }

        let p = parser.push_byte_raw(raw_bytes[13]);
        assert!(p.is_ok());
        let raw_packet = p.ok().unwrap();
        assert_eq!(raw_packet.len(), raw_bytes.len());

        assert_eq!(raw_packet.payload().len(), LinkStatistics::SERIALIZED_LEN);
        assert_eq!(
            raw_packet.raw_packet_type(),
            PacketType::LinkStatistics as u8
        );

        let data = &raw_packet.payload().try_into().unwrap();
        let ls = LinkStatistics::from_bytes(data);
        let p = Packet::parse(&raw_packet).unwrap();
        assert_eq!(Packet::LinkStatistics(ls.clone()), p);

        assert_eq!(ls.uplink_rssi_1, 16);
    }

    #[test]
    fn test_parsing() {
        let raw_bytes: [u8; 40] = [
            0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252, 0xC8, 24, 0x16, 0xE0, 0x03,
            0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01, 0x0B, 0xF8, 0xC0,
            0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 103,
        ];
        let mut parser = CrsfParser::new();
        let results: std::vec::Vec<Result<Packet, CrsfError>> =
            parser.iter_packets(&raw_bytes).collect();

        let expected = [
            992, 992, 352, 992, 352, 352, 352, 352, 352, 352, 992, 992, 0, 0, 0, 0,
        ];

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert_eq!(
            Packet::RCChannels(RcChannelsPacked(expected)),
            results[1].clone().ok().unwrap()
        );
    }
}
