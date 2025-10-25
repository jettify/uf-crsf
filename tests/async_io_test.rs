#![cfg(feature = "async")]
#![cfg(test)]
extern crate std;

use uf_crsf::async_io::write_packet;
use uf_crsf::packets::{LinkStatistics, Packet, PacketAddress};
use uf_crsf::parser::CrsfParser;
use uf_crsf::CrsfStreamError;

async fn build_link_statistics_packet_bytes() -> std::vec::Vec<u8> {
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
    write_packet(&mut buffer, PacketAddress::FlightController, &packet)
        .await
        .unwrap();
    buffer
}

#[tokio::test]
async fn test_write_and_read_packet_async() {
    let packet_bytes = build_link_statistics_packet_bytes().await;

    // Mock reader
    let mut reader = &packet_bytes[..];

    // Parser
    let mut parser = CrsfParser::new();
    let result = parser.read_packet(&mut reader).await;

    assert!(result.is_ok());
    let parsed_packet = result.unwrap();

    if let Packet::LinkStatistics(stats) = parsed_packet {
        assert_eq!(stats.uplink_rssi_1, 10);
        assert_eq!(stats.uplink_link_quality, 95);
        assert_eq!(stats.downlink_link_quality, 98);
    } else {
        panic!("Incorrect packet type");
    }
}

#[tokio::test]
async fn test_read_packet_async_with_no_data() {
    let mut reader = &[][..];
    let mut parser = CrsfParser::new();
    let result = parser.read_packet(&mut reader).await;
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[tokio::test]
async fn test_read_packet_async_with_incomplete_data() {
    let packet_bytes = build_link_statistics_packet_bytes().await;
    let mut reader = &packet_bytes[..packet_bytes.len() - 1];
    let mut parser = CrsfParser::new();
    let result = parser.read_packet(&mut reader).await;
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[tokio::test]
async fn test_read_packet_async_with_garbage() {
    let garbage = [0x01, 0x02, 0x03];
    let mut reader = &garbage[..];
    let mut parser = CrsfParser::new();
    let result = parser.read_packet(&mut reader).await;
    // We expect an InvalidSync error because the first byte is not a valid sync byte.
    assert!(matches!(result, Err(CrsfStreamError::InvalidSync(_))));
}
