#![no_std]

pub mod constants;
pub mod error;
pub mod packets;
pub mod parser;

pub use error::{CrsfError, CrsfParsingError};
pub use packets::{Packet, PacketAddress, PacketType};
pub use parser::{CrsfParser, RawCrsfPacket};

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::parser::ParseResult;
    use packets::{LinkStatistics, RcChannelsPacked};

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

        assert_eq!(raw_packet.payload().len(), LinkStatistics::SERIALIZED_LEN);
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

    #[test]
    fn test_raw_packet_iterator() {
        let raw_bytes: [u8; 40] = [
            0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252, 0xC8, 24, 0x16, 0xE0, 0x03,
            0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01, 0x0B, 0xF8, 0xC0,
            0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 103,
        ];
        let mut parser = CrsfParser::new();
        let results: std::vec::Vec<Result<&[u8], CrsfError>> =
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
        let results: std::vec::Vec<Result<&[u8], CrsfError>> =
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
