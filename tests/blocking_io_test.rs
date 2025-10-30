#![cfg(feature = "embedded_io")]
#![cfg(test)]
extern crate std;

use uf_crsf::blocking_io::write_packet_blocking;
use uf_crsf::packets::{LinkStatistics, Packet, PacketAddress};
use uf_crsf::parser::CrsfParser;
use uf_crsf::CrsfStreamError;

fn build_link_statistics_packet_bytes() -> std::vec::Vec<u8> {
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
    let mut buffer = std::vec::Vec::new();
    write_packet_blocking(&mut buffer, PacketAddress::FlightController, &packet).unwrap();
    buffer
}

#[test]
fn test_write_and_read_packet_blocking() {
    let packet_bytes = build_link_statistics_packet_bytes();

    // Mock reader
    let mut reader = &packet_bytes[..];

    let mut parser = CrsfParser::new();
    let result = parser.read_packet_blocking(&mut reader);

    let parsed_packet = result.unwrap();

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
    assert!(matches!(parsed_packet, Packet::LinkStatistics(p) if p == packet));
}

#[test]
fn test_read_packet_blocking_with_no_data() {
    let mut reader = &[][..];
    let mut parser = CrsfParser::new();
    let result = parser.read_packet_blocking(&mut reader);
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[test]
fn test_read_packet_blocking_with_incomplete_data() {
    let packet_bytes = build_link_statistics_packet_bytes();
    let mut reader = &packet_bytes[..packet_bytes.len() - 1];
    let mut parser = CrsfParser::new();
    let result = parser.read_packet_blocking(&mut reader);
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[test]
fn test_read_packet_blocking_with_garbage() {
    let garbage = [0x01, 0x02, 0x03];
    let mut reader = &garbage[..];
    let mut parser = CrsfParser::new();
    let result = parser.read_packet_blocking(&mut reader);
    // We expect an InvalidSync error because the first byte is not a valid sync byte.
    assert!(matches!(result, Err(CrsfStreamError::InvalidSync(_))));
}
