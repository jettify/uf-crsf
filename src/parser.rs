use crate::{
    constants,
    error::CrsfError,
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
    Error(CrsfError),
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
                    ParseResult::Error(CrsfError::InvalidSync)
                }
            }
            State::AwaitingLenth => {
                let n = byte as usize + 2;

                if !(constants::CRSF_MIN_PACKET_SIZE..constants::CRSF_MAX_PACKET_SIZE).contains(&n)
                {
                    self.reset();
                    return ParseResult::Error(CrsfError::InvalidPacketLength);
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
                    return ParseResult::Error(CrsfError::InvalidCrc {
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
                Err(e) => ParseResult::Error(CrsfError::ParsingError(e)),
            },
            ParseResult::Incomplete => ParseResult::Incomplete,
            ParseResult::Error(e) => ParseResult::Error(e),
        }
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.state = State::AwaitingSync
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
}

pub struct RawPacketIterator<'a, 'b> {
    parser: &'a mut CrsfParser,
    buffer: &'b [u8],
    pos: usize,
}

impl<'a, 'b> Iterator for RawPacketIterator<'a, 'b> {
    type Item = Result<&'b [u8], CrsfError>;

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
    type Item = Result<Packet, CrsfError>;

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
