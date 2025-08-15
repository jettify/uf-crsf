#![cfg(test)]
extern crate std;

use uf_crsf::packets::{LinkStatistics, Packet, PacketAddress};
use uf_crsf::parser::CrsfParser;
use uf_crsf::write_packet_to_buffer;
use uf_crsf::CrsfStreamError;

fn build_link_statistics_packet() -> ([u8; 64], usize) {
    let packet = LinkStatistics {
        uplink_rssi_1: 10,
        uplink_rssi_2: 20,
        uplink_link_quality: 95,
        uplink_snr: -80,
        active_antenna: 1,
        rf_mode: 2,
        uplink_tx_power: 3,
        downlink_rssi: 30,
        downlink_link_quality: 98,
        downlink_snr: -75,
    };
    let mut buffer = [0u8; 64];
    let bytes_written =
        write_packet_to_buffer(&mut buffer, PacketAddress::FlightController, &packet).unwrap();
    (buffer, bytes_written)
}

#[test]
fn test_stream_with_back_to_back_packets() {
    let (packet_buffer1, len1) = build_link_statistics_packet();
    let (packet_buffer2, len2) = build_link_statistics_packet();

    let mut stream = std::vec::Vec::new();
    stream.extend_from_slice(&packet_buffer1[..len1]);
    stream.extend_from_slice(&packet_buffer2[..len2]);

    let mut parser = CrsfParser::new();
    let packets: std::vec::Vec<Result<Packet, CrsfStreamError>> =
        parser.iter_packets(&stream).collect();

    assert_eq!(packets.len(), 2);
    assert!(packets[0].is_ok());
    assert!(packets[1].is_ok());
}

#[test]
fn test_stream_with_interspersed_garbage() {
    let (packet_buffer, len) = build_link_statistics_packet();
    let garbage = [0x01, 0x02, 0x03, 0x04, 0x05];

    let mut stream = std::vec::Vec::new();
    stream.extend_from_slice(&garbage);
    stream.extend_from_slice(&packet_buffer[..len]);
    stream.extend_from_slice(&garbage);
    stream.extend_from_slice(&packet_buffer[..len]);
    stream.extend_from_slice(&garbage);

    let mut parser = CrsfParser::new();
    // We expect errors from the garbage, but the iterator should recover
    let packets: std::vec::Vec<Packet> = parser
        .iter_packets(&stream)
        .filter_map(Result::ok)
        .collect();

    assert_eq!(packets.len(), 2);
}

#[test]
fn test_stream_with_partial_first_packet() {
    let (packet_buffer1, len1) = build_link_statistics_packet();
    let (packet_buffer2, len2) = build_link_statistics_packet();

    let mut stream = std::vec::Vec::new();
    // Start with the last half of the first packet (which is invalid)
    stream.extend_from_slice(&packet_buffer1[len1 / 2..len1]);
    // Then a full, valid packet
    stream.extend_from_slice(&packet_buffer2[..len2]);

    let mut parser = CrsfParser::new();
    let packets: std::vec::Vec<Packet> = parser
        .iter_packets(&stream)
        .filter_map(Result::ok)
        .collect();

    assert_eq!(packets.len(), 1);
}
