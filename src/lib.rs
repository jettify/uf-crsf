#![no_std]
#![forbid(unsafe_code)]
mod packet;
use packet::PacketType;
use packet::RawCrsfPacket;

pub mod constants {
    pub const CRSF_SYNC_BYTE: u8 = 0xC8;
    pub const CRSF_MAX_PACKET_SIZE: usize = 64;
    pub const CRSF_MIN_PACKET_SIZE: usize = 4; // Header (1) + Type (1) + CRC (1) + Payload (min 1)
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
    InvalidCrc(u8, u8),
    NeedMoreData(usize),
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
                let n = byte as usize + constants::CRSF_PACKET_HEADER_LEN;

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
                    return Err(CrsfError::InvalidCrc(calculated_crc, packet_crc));
                }

                let start = 0;
                let end = self.position + 1;
                self.reset();
                let bytes = &self.buffer[start..end];
                Ok(RawCrsfPacket { bytes })
            }
            State::Reading(n) => {
                self.position += 1;
                self.buffer[self.position] = byte;
                Err(CrsfError::NeedMoreData(n - self.position))
            }
        }
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.state = State::AwaitingHead
    }
}

pub struct PacketIterator<'a, 'b> {
    parser: &'a mut CrsfParser,
    remaining_data: &'b [u8],
}

impl Default for CrsfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use crate::packet::{LinkStatistics, Packet};

    use super::*;

    #[test]
    fn test_construction() {
        let raw_bytes: [u8; 14] = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
        let mut parser = CrsfParser::new();

        for b in &raw_bytes[0..raw_bytes.len() - 1] {
            std::dbg!(b);
            assert!(std::dbg!(parser.push_byte_raw(*b)).is_err());
        }

        let p = std::dbg!(parser.push_byte_raw(raw_bytes[13]));
        assert!(p.is_ok());
        let raw_packet = p.ok().unwrap();
        std::dbg!(raw_packet.bytes);
        assert_eq!(raw_packet.bytes.len(), raw_bytes.len());
        assert_eq!(raw_packet.bytes, raw_bytes);

        assert_eq!(raw_packet.payload().len(), LinkStatistics::SERIALIZED_LEN);
        assert_eq!(raw_packet.packet_type(), PacketType::LinkStatistics);

        let data = &raw_packet.payload().try_into().unwrap();
        let ls = LinkStatistics::from_bytes(data);
        let p = Packet::parse(&raw_packet);
        assert_eq!(Packet::LinkStatistics(ls.clone()), p);

        assert_eq!(ls.uplink_rssi_1, 16);
    }
}
