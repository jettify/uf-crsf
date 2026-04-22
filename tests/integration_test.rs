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

fn build_link_statistics_packet_for(dest: PacketAddress, packet: &LinkStatistics) -> Vec<u8> {
    let mut buffer = [0u8; 64];
    let len = write_packet_to_buffer(&mut buffer, dest, packet).unwrap();
    buffer[..len].to_vec()
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

#[test]
fn test_stream_recovers_after_incomplete_frame_across_chunks() {
    let mut parser = CrsfParser::new();

    let chunks: Vec<Vec<u8>> = vec![
        vec![
            238, 24, 22, 34, 5, 95, 43, 90, 209, 138, 77, 181, 2, 124, 176, 104, 5, 248, 192, 7,
            62, 240, 129, 15, 124, 208,
        ],
        vec![
            238, 24, 22, 33, 5, 95, 43, 90, 209, 138, 77, 181, 2, 124, 210, 104, 5, 248, 192, 7,
            62, 240, 129, 15, 124, 246,
        ],
        vec![
            24, 22, 33, 5, 95, 43, 90, 209, 10, 78, 181, 2, 124, 219, 104, 5, 248, 192, 7, 62, 240,
            129, 15, 124,
        ],
        vec![
            238, 24, 22, 33, 5, 95, 43, 90, 209, 10, 78, 181, 2, 124, 236, 104, 5, 248, 192, 7, 62,
            240, 129, 15, 124, 136,
        ],
        vec![
            238, 24, 22, 33, 5, 95, 43, 90, 209, 10, 78, 181, 2, 124, 36, 105, 5, 248, 192, 7, 62,
            240, 129, 15, 124, 89,
        ],
        vec![
            238, 24, 22, 33, 5, 95, 43, 90, 209, 10, 78, 181, 2, 124, 36, 105, 5, 248, 192, 7, 62,
            240, 129, 15, 124, 89,
        ],
        vec![
            238, 24, 22, 33, 5, 95, 43, 90, 209, 10, 78, 181, 2, 124, 36, 105, 5, 248, 192, 7, 62,
            240, 129, 15, 124, 89,
        ],
    ];

    let mut parsed_in_first_two_chunks = 0usize;
    let mut parsed_after_incomplete_chunk = 0usize;

    for (index, chunk) in chunks.iter().enumerate() {
        for item in parser.iter_packets(chunk) {
            if item.is_ok() {
                if index < 2 {
                    parsed_in_first_two_chunks += 1;
                } else if index >= 3 {
                    parsed_after_incomplete_chunk += 1;
                }
            }
        }
    }

    assert_eq!(parsed_in_first_two_chunks, 2);
    assert!(
        parsed_after_incomplete_chunk > 0,
        "parser did not recover after incomplete frame"
    );
}

#[test]
fn test_false_sync_at_crc_boundary_recovery() {
    let packet = LinkStatistics {
        uplink_rssi_1: 1,
        uplink_rssi_2: 2,
        uplink_link_quality: 3,
        uplink_snr: -4,
        active_antenna: 5,
        rf_mode: 6,
        uplink_tx_power: 7,
        downlink_rssi: 8,
        downlink_link_quality: 9,
        downlink_snr: -10,
    };
    let frame = build_link_statistics_packet_for(PacketAddress::Transmitter, &packet);

    // False frame candidate:
    // 0xC0 is a valid sync, length 4 is plausible, then 0xEE is used as fake CRC.
    // 0xEE is also the next real frame sync and must be preserved for recovery.
    let mut stream = vec![0xC0, 0x04, 0x16, 0x00, 0x00, 0xEE];
    stream.extend_from_slice(&frame[1..]);

    let mut parser = CrsfParser::new();
    let packets: Vec<Packet> = parser
        .iter_packets(&stream)
        .filter_map(Result::ok)
        .collect();

    assert_eq!(packets.len(), 1);
}

#[test]
fn test_invalid_length_byte_reused_as_sync() {
    let packet = LinkStatistics {
        uplink_rssi_1: 11,
        uplink_rssi_2: 22,
        uplink_link_quality: 33,
        uplink_snr: -44,
        active_antenna: 1,
        rf_mode: 2,
        uplink_tx_power: 3,
        downlink_rssi: 44,
        downlink_link_quality: 55,
        downlink_snr: -66,
    };
    let frame = build_link_statistics_packet_for(PacketAddress::Transmitter, &packet);

    // 0xC8 starts a frame, then 0xEE is an invalid length byte but valid sync.
    // Parser should immediately reinterpret 0xEE as sync and parse the frame.
    let mut stream = vec![
        PacketAddress::FlightController as u8,
        PacketAddress::Transmitter as u8,
    ];
    stream.extend_from_slice(&frame[1..]);

    let mut parser = CrsfParser::new();
    let packets: Vec<Packet> = parser
        .iter_packets(&stream)
        .filter_map(Result::ok)
        .collect();

    assert_eq!(packets.len(), 1);
}

#[test]
fn test_state_persistence_across_iter_packets_calls() {
    let p1 = build_link_statistics_packet_for(
        PacketAddress::Transmitter,
        &LinkStatistics {
            uplink_rssi_1: 31,
            uplink_rssi_2: 32,
            uplink_link_quality: 33,
            uplink_snr: -34,
            active_antenna: 1,
            rf_mode: 2,
            uplink_tx_power: 3,
            downlink_rssi: 35,
            downlink_link_quality: 36,
            downlink_snr: -37,
        },
    );
    let p2 = build_link_statistics_packet_for(
        PacketAddress::Transmitter,
        &LinkStatistics {
            uplink_rssi_1: 41,
            uplink_rssi_2: 42,
            uplink_link_quality: 43,
            uplink_snr: -44,
            active_antenna: 1,
            rf_mode: 2,
            uplink_tx_power: 3,
            downlink_rssi: 45,
            downlink_link_quality: 46,
            downlink_snr: -47,
        },
    );

    let mut parser = CrsfParser::new();
    let first_call_packets: Vec<Packet> = parser.iter_packets(&p1).filter_map(Result::ok).collect();
    assert_eq!(first_call_packets.len(), 1);

    // Leave parser in a bad candidate frame that would consume next sync on old behavior.
    let middle_chunk = [0xC0, 0x04, 0x16, 0x00, 0x00];
    let middle_call_packets: Vec<Packet> = parser
        .iter_packets(&middle_chunk)
        .filter_map(Result::ok)
        .collect();
    assert!(middle_call_packets.is_empty());

    let second_call_packets: Vec<Packet> =
        parser.iter_packets(&p2).filter_map(Result::ok).collect();
    assert_eq!(second_call_packets.len(), 1);
}

#[test]
fn test_address_bytes_in_payload_do_not_prevent_recovery() {
    let packet = LinkStatistics {
        uplink_rssi_1: 0xC0,
        uplink_rssi_2: 0xC8,
        uplink_link_quality: 0xEE,
        uplink_snr: -112, // 0x90
        active_antenna: 0x91,
        rf_mode: 0x92,
        uplink_tx_power: 0x93,
        downlink_rssi: 0x94,
        downlink_link_quality: 0x95,
        downlink_snr: -106, // 0x96
    };
    let frame = build_link_statistics_packet_for(PacketAddress::FlightController, &packet);

    let chunk1 = &frame[..frame.len() / 2];
    let chunk2 = &[24, 22, 33, 5, 95, 43, 90, 209, 10, 78];
    let chunk3 = &frame;

    let mut parser = CrsfParser::new();
    let mut ok_count = 0usize;
    for chunk in [chunk1, chunk2, chunk3] {
        for item in parser.iter_packets(chunk) {
            if item.is_ok() {
                ok_count += 1;
            }
        }
    }

    assert!(ok_count >= 1);
}

#[test]
fn test_chunk_boundary_all_split_points() {
    let (packet_buffer, len) = build_link_statistics_packet();
    let mut stream = Vec::new();
    stream.extend_from_slice(&packet_buffer[..len]);
    stream.extend_from_slice(&packet_buffer[..len]);
    stream.extend_from_slice(&packet_buffer[..len]);

    for split in 1..stream.len() {
        let mut parser = CrsfParser::new();
        let mut count = 0usize;

        for item in parser.iter_packets(&stream[..split]) {
            if item.is_ok() {
                count += 1;
            }
        }
        for item in parser.iter_packets(&stream[split..]) {
            if item.is_ok() {
                count += 1;
            }
        }

        assert_eq!(count, 3, "failed split at index {split}");
    }
}
