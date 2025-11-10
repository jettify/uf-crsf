use crate::{
    constants,
    error::CrsfStreamError,
    packets::{Packet, PacketAddress},
};
use crc::Crc;
use num_enum::TryFromPrimitive;

#[derive(Debug, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

const CRC8_DVB_S2: Crc<u8> = Crc::<u8>::new(&crc::CRC_8_DVB_S2);

impl CrsfParser {
    pub fn new() -> Self {
        Self {
            buffer: [0; constants::CRSF_MAX_PACKET_SIZE],
            state: State::AwaitingSync,
            position: 0,
        }
    }

    pub fn push_byte_raw(
        &mut self,
        byte: u8,
    ) -> Result<Option<RawCrsfPacket<'_>>, CrsfStreamError> {
        match self.state {
            State::AwaitingSync => {
                if PacketAddress::try_from_primitive(byte).is_ok() {
                    self.position = 0;
                    self.buffer[self.position] = byte;
                    self.state = State::AwaitingLenth;
                    Ok(None)
                } else {
                    self.state = State::AwaitingSync;
                    Err(CrsfStreamError::InvalidSync(byte))
                }
            }
            State::AwaitingLenth => {
                let n = byte as usize + 2;

                if !(constants::CRSF_MIN_PACKET_SIZE..constants::CRSF_MAX_PACKET_SIZE).contains(&n)
                {
                    self.reset();
                    return Err(CrsfStreamError::InvalidPacketLength(byte));
                }
                self.position = 1;
                self.buffer[self.position] = byte;
                self.state = State::Reading(n - 1);
                Ok(None)
            }
            State::Reading(n) => {
                self.position += 1;
                self.buffer[self.position] = byte;
                if self.position == n - 1 {
                    self.state = State::AwaitingCrc;
                }
                Ok(None)
            }
            State::AwaitingCrc => {
                self.position += 1;
                self.buffer[self.position] = byte;

                let mut digest = CRC8_DVB_S2.digest();
                digest.update(&self.buffer[2..self.position]);
                let calculated_crc = digest.finalize();
                let packet_crc = self.buffer[self.position];

                if calculated_crc != packet_crc {
                    self.reset();
                    return Err(CrsfStreamError::InvalidCrc {
                        calculated_crc,
                        packet_crc,
                    });
                }
                let start = 0;
                let end = self.position + 1;
                self.reset();
                let bytes = &self.buffer[start..end];
                match RawCrsfPacket::new(bytes) {
                    None => Err(CrsfStreamError::InputBufferTooSmall),
                    Some(packet) => Ok(Some(packet)),
                }
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

    pub fn push_byte(&mut self, byte: u8) -> Result<Option<Packet>, CrsfStreamError> {
        match self.push_byte_raw(byte) {
            Ok(Some(raw_packet)) => match Packet::parse(&raw_packet) {
                Ok(packet) => Ok(Some(packet)),
                Err(e) => Err(CrsfStreamError::ParsingError(e)),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
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

/// Represents a valid, but unparsed, CRSF packet.
///
/// This struct is a zero-copy view into a byte buffer that has been validated
/// to contain a complete CRSF packet, including the sync byte, length, type,
/// payload, and CRC. It provides methods to access the different parts of the
/// packet without parsing the payload itself.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawCrsfPacket<'a> {
    bytes: &'a [u8],
}

impl<'a> RawCrsfPacket<'a> {
    /// Creates a new `RawCrsfPacket` from a byte slice.
    ///
    /// Returns `None` if the slice is shorter than the minimum possible
    /// CRSF packet length (4 bytes).
    pub fn new(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() >= 4 {
            Some(Self { bytes })
        } else {
            None
        }
    }

    /// Returns the destination address byte of the packet.
    pub fn dst_addr(&self) -> u8 {
        self.bytes[0]
    }

    /// Returns the raw packet type byte.
    pub fn raw_packet_type(&self) -> u8 {
        self.bytes[2]
    }

    /// Returns a slice representing the packet's payload.
    ///
    /// The payload does not include the CRSF framing (destination, size, type, CRC).
    pub fn payload(&self) -> &[u8] {
        &self.bytes[3..self.bytes.len() - 1]
    }

    /// Returns the CRC check byte of the packet.
    #[expect(clippy::missing_panics_doc, reason = "infallible")]
    pub fn crc(&self) -> u8 {
        *self.bytes.last().expect("infallible due to length check")
    }

    /// Returns the total length of the packet in bytes, including the framing.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns `true` if the packet has a length of zero.
    ///
    /// Note: A valid CRSF packet should not be empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
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
                Ok(Some(packet)) => return Some(Ok(packet)),
                Ok(None) => (),
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::packets::{
        write_packet_to_buffer, CrsfPacket, LinkStatistics, PacketAddress, PacketType,
        RcChannelsPacked,
    };

    #[test]
    fn test_construction() {
        let raw_bytes: [u8; 14] = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
        let mut parser = CrsfParser::new();

        for b in &raw_bytes[0..raw_bytes.len() - 1] {
            let result = parser.push_byte_raw(*b);
            assert!(matches!(result, Ok(None)));
        }

        let raw_packet_result = parser.push_byte_raw(raw_bytes[13]);
        let raw_packet = raw_packet_result
            .expect("Failed to get raw packet result")
            .expect("Expected a complete raw packet, but got None");

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
    fn test_raw_to_full_packet_conversion() {
        let link_stats_packet = LinkStatistics {
            uplink_rssi_1: 16,
            uplink_rssi_2: 19,
            uplink_link_quality: 99,
            uplink_snr: 51 as i8,
            active_antenna: 1,
            rf_mode: 2,
            uplink_tx_power: 3,
            downlink_rssi: 8,
            downlink_link_quality: 88,
            downlink_snr: 48 as i8,
        };

        // Serialize it into a buffer
        let mut buffer = [0u8; 64];
        let bytes_written = write_packet_to_buffer(
            &mut buffer,
            PacketAddress::FlightController,
            &link_stats_packet,
        )
        .unwrap();
        let raw_bytes = &buffer[..bytes_written];

        let mut parser = CrsfParser::new();

        // 1. Parse raw bytes to get a RawCrsfPacket
        let mut raw_packet_result = Ok(None);
        for &byte in raw_bytes {
            raw_packet_result = parser.push_byte_raw(byte);
            if let Ok(Some(_)) = &raw_packet_result {
                break;
            }
        }
        let raw_packet = raw_packet_result
            .expect("Failed to get raw packet result")
            .expect("Expected a complete raw packet, but got None");

        let packet = Packet::parse(&raw_packet).expect("Failed to parse raw packet into a Packet");

        // Verify the resulting packet
        assert!(matches!(packet, Packet::LinkStatistics(_)));
        if let Packet::LinkStatistics(stats) = packet {
            assert_eq!(stats, link_stats_packet)
        }
    }
}
