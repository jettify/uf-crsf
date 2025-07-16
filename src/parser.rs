use crate::{
    constants,
    error::CrsfStreamError,
    packets::{Packet, PacketAddress},
};
use num_enum::TryFromPrimitive;

#[derive(Debug, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum State {
    #[default]
    AwaitingSync,
    AwaitingLenth,
    Reading(usize),
    AwaitingCrc,
}

#[derive(Debug)]
pub struct CrsfParser {
    buffer: [u8; constants::CRSF_MAX_PACKET_SIZE],
    state: State,
    position: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseResult<T> {
    Complete(T),
    Incomplete,
    Error(CrsfStreamError),
}

impl CrsfParser {
    pub fn new() -> Self {
        Self {
            buffer: [0; constants::CRSF_MAX_PACKET_SIZE],
            state: State::AwaitingSync,
            position: 0,
        }
    }

    pub fn push_byte_raw(&mut self, byte: u8) -> ParseResult<RawCrsfPacket<'_>> {
        match self.state {
            State::AwaitingSync => {
                if PacketAddress::try_from_primitive(byte).is_ok() {
                    self.position = 0;
                    self.buffer[self.position] = byte;
                    self.state = State::AwaitingLenth;
                    ParseResult::Incomplete
                } else {
                    self.state = State::AwaitingSync;
                    ParseResult::Error(CrsfStreamError::InvalidSync)
                }
            }
            State::AwaitingLenth => {
                let n = byte as usize + 2;

                if !(constants::CRSF_MIN_PACKET_SIZE..constants::CRSF_MAX_PACKET_SIZE).contains(&n)
                {
                    self.reset();
                    return ParseResult::Error(CrsfStreamError::InvalidPacketLength);
                }
                self.position = 1;
                self.buffer[self.position] = byte;
                self.state = State::Reading(n - 1);
                ParseResult::Incomplete
            }
            State::Reading(n) => {
                self.position += 1;
                self.buffer[self.position] = byte;
                if self.position == n - 1 {
                    self.state = State::AwaitingCrc;
                }
                ParseResult::Incomplete
            }
            State::AwaitingCrc => {
                self.position += 1;
                self.buffer[self.position] = byte;

                let crc8_dvb_s2 = crc::Crc::<u8>::new(&crc::CRC_8_DVB_S2);
                let mut digest = crc8_dvb_s2.digest();
                digest.update(&self.buffer[2..self.position]);
                let calculated_crc = digest.finalize();
                let packet_crc = self.buffer[self.position];

                if calculated_crc != packet_crc {
                    self.reset();
                    return ParseResult::Error(CrsfStreamError::InvalidCrc {
                        calculated_crc,
                        packet_crc,
                    });
                }
                let start = 0;
                let end = self.position + 1;
                self.reset();
                let bytes = &self.buffer[start..end];
                ParseResult::Complete(RawCrsfPacket::new(bytes).unwrap())
            }
        }
    }

    pub fn iter_packets<'a, 'b>(&'a mut self, buffer: &'b [u8]) -> PacketIterator<'a, 'b> {
        PacketIterator {
            parser: self,
            buffer,
            pos: 0,
        }
    }

    pub fn iter_packets_raw<'a, 'b>(&'a mut self, buffer: &'b [u8]) -> RawPacketIterator<'a, 'b> {
        RawPacketIterator {
            parser: self,
            buffer,
            pos: 0,
        }
    }

    pub fn push_byte(&mut self, byte: u8) -> ParseResult<Packet> {
        match self.push_byte_raw(byte) {
            ParseResult::Complete(raw_packet) => match Packet::parse(&raw_packet) {
                Ok(packet) => ParseResult::Complete(packet),
                Err(e) => ParseResult::Error(CrsfStreamError::ParsingError(e)),
            },
            ParseResult::Incomplete => ParseResult::Incomplete,
            ParseResult::Error(e) => ParseResult::Error(e),
        }
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.state = State::AwaitingSync;
    }
}

impl Default for CrsfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawCrsfPacket<'a> {
    bytes: &'a [u8],
}

impl<'a> RawCrsfPacket<'a> {
    pub fn new(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() >= 4 {
            Some(Self { bytes })
        } else {
            None
        }
    }

    pub fn dst_addr(&self) -> u8 {
        self.bytes[0]
    }
    pub fn raw_packet_type(&self) -> u8 {
        // XXX
        self.bytes[2]
    }

    pub fn payload(&self) -> &[u8] {
        // XXX
        &self.bytes[3..self.bytes.len() - 1]
    }
    pub fn crc(&self) -> u8 {
        *self.bytes.last().unwrap()
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

pub struct RawPacketIterator<'a, 'b> {
    parser: &'a mut CrsfParser,
    buffer: &'b [u8],
    pos: usize,
}

impl<'b> Iterator for RawPacketIterator<'_, 'b> {
    type Item = Result<&'b [u8], CrsfStreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buffer.len() {
            let byte = self.buffer[self.pos];
            let result = self.parser.push_byte_raw(byte);
            self.pos += 1;

            match result {
                ParseResult::Complete(raw_packet) => {
                    let packet_len = raw_packet.len();
                    let start_index = self.pos - packet_len;
                    return Some(Ok(&self.buffer[start_index..self.pos]));
                }
                ParseResult::Incomplete => (),
                ParseResult::Error(err) => return Some(Err(err)),
            }
        }
        None
    }
}

pub struct PacketIterator<'a, 'b> {
    parser: &'a mut CrsfParser,
    buffer: &'b [u8],
    pos: usize,
}

impl Iterator for PacketIterator<'_, '_> {
    type Item = Result<Packet, CrsfStreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buffer.len() {
            let byte = self.buffer[self.pos];
            self.pos += 1;

            match self.parser.push_byte(byte) {
                ParseResult::Complete(packet) => return Some(Ok(packet)),
                ParseResult::Incomplete => (),
                ParseResult::Error(err) => return Some(Err(err)),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::packets::{CrsfPacket, LinkStatistics, PacketType, RcChannelsPacked};
    use crate::parser::ParseResult;

    #[test]
    fn test_construction() {
        let raw_bytes: [u8; 14] = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
        let mut parser = CrsfParser::new();

        for b in &raw_bytes[0..raw_bytes.len() - 1] {
            let result = parser.push_byte_raw(*b);
            assert!(matches!(result, ParseResult::Incomplete));
        }

        let p = parser.push_byte_raw(raw_bytes[13]);
        let raw_packet = match p {
            ParseResult::Complete(packet) => packet,
            _ => panic!("Expected complete packet"),
        };
        assert_eq!(raw_packet.len(), raw_bytes.len());

        assert_eq!(raw_packet.payload().len(), LinkStatistics::MIN_PAYLOAD_SIZE);
        assert_eq!(
            raw_packet.raw_packet_type(),
            PacketType::LinkStatistics as u8
        );

        let data = &raw_packet.payload();
        let ls = LinkStatistics::from_bytes(data).unwrap();
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
        let results: std::vec::Vec<Result<Packet, CrsfStreamError>> =
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

    #[test]
    fn test_raw_packet_iterator() {
        let raw_bytes: [u8; 40] = [
            0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252, 0xC8, 24, 0x16, 0xE0, 0x03,
            0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01, 0x0B, 0xF8, 0xC0,
            0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 103,
        ];
        let mut parser = CrsfParser::new();
        let results: std::vec::Vec<Result<&[u8], CrsfStreamError>> =
            parser.iter_packets_raw(&raw_bytes).collect();

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert_eq!(results[0].unwrap(), &raw_bytes[0..14]);
        assert!(results[1].is_ok());
        assert_eq!(results[1].unwrap(), &raw_bytes[14..40]);
    }

    #[test]
    fn test_raw_iterator_and_manual_parse() {
        let raw_bytes: [u8; 40] = [
            // Packet 1: LinkStatistics
            0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252,
            // Packet 2: RCChannels
            0xC8, 24, 0x16, 0xE0, 0x03, 0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60,
            0x01, 0x0B, 0xF8, 0xC0, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 103,
        ];
        let mut parser = CrsfParser::new();
        let results: std::vec::Vec<Result<&[u8], CrsfStreamError>> =
            parser.iter_packets_raw(&raw_bytes).collect();

        assert_eq!(results.len(), 2);

        // First packet
        let raw_packet_bytes_1 = results[0].as_ref().unwrap();
        assert_eq!(*raw_packet_bytes_1, &raw_bytes[0..14]);
        let raw_packet_1 = RawCrsfPacket::new(raw_packet_bytes_1).unwrap();
        let packet_1 = Packet::parse(&raw_packet_1).unwrap();

        // Manually create expected packet to compare
        let payload_1: &[u8; 10] = raw_packet_1.payload().try_into().unwrap();
        let ls = LinkStatistics::from_bytes(payload_1).unwrap();
        let expected_packet_1 = Packet::LinkStatistics(ls);
        assert_eq!(expected_packet_1, packet_1);

        // Second packet
        let raw_packet_bytes_2 = results[1].as_ref().unwrap();
        assert_eq!(*raw_packet_bytes_2, &raw_bytes[14..40]);
        let raw_packet_2 = RawCrsfPacket::new(raw_packet_bytes_2).unwrap();
        let packet_2 = Packet::parse(&raw_packet_2).unwrap();

        // Manually create expected packet to compare
        let expected_channels = [
            992, 992, 352, 992, 352, 352, 352, 352, 352, 352, 992, 992, 0, 0, 0, 0,
        ];
        let expected_packet_2 = Packet::RCChannels(RcChannelsPacked(expected_channels));
        assert_eq!(expected_packet_2, packet_2);
    }
}
