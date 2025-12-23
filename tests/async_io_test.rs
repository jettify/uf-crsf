#![cfg(feature = "embedded_io_async")]
#![cfg(test)]
extern crate std;

use uf_crsf::async_io::{write_packet, AsyncCrsfReader};
use uf_crsf::packets::{LinkStatistics, Packet, PacketAddress};
use uf_crsf::CrsfStreamError;

async fn build_link_statistics_packet_bytes(uplink_rssi_1: u8) -> std::vec::Vec<u8> {
    let packet = LinkStatistics {
        uplink_rssi_1: uplink_rssi_1,
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
    let packet_bytes = build_link_statistics_packet_bytes(10).await;

    // Mock reader
    let reader = &packet_bytes[..];

    // Parser
    let mut reader = AsyncCrsfReader::new(reader);
    let result = reader.read_packet().await;

    assert!(result.is_ok());
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

#[tokio::test]
async fn test_read_packet_async_with_no_data() {
    let mut reader = AsyncCrsfReader::new(&[][..]);
    let result = reader.read_packet().await;
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[tokio::test]
async fn test_read_packet_async_with_incomplete_data() {
    let packet_bytes = build_link_statistics_packet_bytes(10).await;
    let mut reader = AsyncCrsfReader::new(&packet_bytes[..packet_bytes.len() - 1]);
    let result = reader.read_packet().await;
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[tokio::test]
async fn test_read_packet_async_with_garbage() {
    let garbage = [0x01, 0x02, 0x03];
    let mut reader = AsyncCrsfReader::new(&garbage[..]);
    let result = reader.read_packet().await;
    // We expect an InvalidSync error because the first byte is not a valid sync byte.
    assert!(matches!(result, Err(CrsfStreamError::InvalidSync(_))));
}

#[tokio::test]
async fn test_read_packet_async_chunked_stream() {
    let packet1_bytes = build_link_statistics_packet_bytes(10).await;
    let packet2_bytes = build_link_statistics_packet_bytes(50).await;

    let mut combined_bytes = std::vec::Vec::new();
    combined_bytes.extend_from_slice(&packet1_bytes);
    combined_bytes.extend_from_slice(&packet2_bytes);

    let mut stream_reader = AsyncCrsfReader::new(&combined_bytes[..]);

    let result1 = stream_reader.read_packet().await;
    let result2 = stream_reader.read_packet().await;

    assert!(result1.is_ok());
    let parsed_packet1 = result1.unwrap();
    let expected_packet1 = LinkStatistics {
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
    assert!(matches!(&parsed_packet1, Packet::LinkStatistics(p) if p == &expected_packet1));

    assert!(
        result2.is_ok(),
        "Second packet parsing failed: {:?}",
        result2.err()
    );
    let parsed_packet2 = result2.unwrap();
    let expected_packet2 = LinkStatistics {
        uplink_rssi_1: 50,
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
    assert!(matches!(parsed_packet2, Packet::LinkStatistics(p) if p == expected_packet2));
}
