#![cfg(feature = "embedded_io_async")]
#![cfg(test)]
extern crate std;

use uf_crsf::async_io::write_packet;
use uf_crsf::async_io::AsyncCrsfReader;
use uf_crsf::packets::LinkStatistics;
use uf_crsf::packets::Packet;
use uf_crsf::packets::PacketAddress;
use uf_crsf::CrsfStreamError;

async fn build_link_statistics_packet_bytes(uplink_rssi_1: u8) -> std::vec::Vec<u8> {
    let packet = LinkStatistics {
        uplink_rssi_1,
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

#[futures_test::test]
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

#[futures_test::test]
async fn test_read_packet_async_with_no_data() {
    let mut reader = AsyncCrsfReader::new(&[][..]);
    let result = reader.read_packet().await;
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[futures_test::test]
async fn test_read_packet_async_with_incomplete_data() {
    let packet_bytes = build_link_statistics_packet_bytes(10).await;
    let mut reader = AsyncCrsfReader::new(&packet_bytes[..packet_bytes.len() - 1]);
    let result = reader.read_packet().await;
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[futures_test::test]
async fn test_read_packet_async_with_garbage() {
    let garbage = [0x01, 0x02, 0x03];
    let mut reader = AsyncCrsfReader::new(&garbage[..]);
    let result = reader.read_packet().await;
    // We expect an InvalidSync error because the first byte is not a valid sync byte.
    assert!(matches!(result, Err(CrsfStreamError::InvalidSync(_))));
}

#[futures_test::test]
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

mod common;

#[futures_test::test]
async fn buffered_results_resume_without_transport_reads() {
    use common::ChunkReader;
    use common::VALID_FRAME;
    use std::cell::Cell;
    use std::rc::Rc;
    let mut invalid_crc = VALID_FRAME;
    invalid_crc[13] ^= 1;
    let invalid_payload = [
        0xC8,
        2,
        0x14,
        crc::Crc::<u8>::new(&crc::CRC_8_DVB_S2).checksum(&[0x14]),
    ];
    let bytes = [
        &[0xFF][..],
        &VALID_FRAME,
        &invalid_crc,
        &invalid_payload,
        &VALID_FRAME,
    ]
    .concat();
    let reads = Rc::new(Cell::new(0));
    let mut reader = AsyncCrsfReader::new(ChunkReader {
        input: &bytes,
        chunk_size: 64,
        reads: reads.clone(),
        fail_on: None,
    });
    assert_eq!(
        reader.read_packet().await,
        Err(CrsfStreamError::InvalidSync(0xFF))
    );
    assert!(matches!(
        reader.read_packet().await,
        Ok(Packet::LinkStatistics(_))
    ));
    assert!(matches!(
        reader.read_packet().await,
        Err(CrsfStreamError::InvalidCrc { .. })
    ));
    let mut recovery_errors = 0;
    loop {
        let result = reader.read_packet().await;
        assert_eq!(reads.get(), 1);
        match result {
            Err(CrsfStreamError::ParsingError(_)) => break,
            Err(
                CrsfStreamError::InvalidSync(_)
                | CrsfStreamError::InvalidPacketLength(_)
                | CrsfStreamError::InvalidCrc { .. },
            ) => recovery_errors += 1,
            other => panic!("expected recovery error or payload error, got {other:?}"),
        }
    }
    assert!(recovery_errors > 0);
    assert!(matches!(
        reader.read_packet().await,
        Ok(Packet::LinkStatistics(_))
    ));
    assert_eq!(reads.get(), 1);
    assert_eq!(
        reader.read_packet().await,
        Err(CrsfStreamError::UnexpectedEof)
    );
    assert_eq!(reads.get(), 2);
}

#[futures_test::test]
async fn split_reads_and_transport_error_preserve_partial_frames() {
    use common::ChunkReader;
    use common::VALID_FRAME;
    use std::cell::Cell;
    use std::rc::Rc;
    let bytes = [VALID_FRAME, VALID_FRAME].concat();
    for chunk_size in [1, 2, 7, 64] {
        let mut reader = AsyncCrsfReader::new(ChunkReader {
            input: &bytes,
            chunk_size,
            reads: Rc::new(Cell::new(0)),
            fail_on: None,
        });
        for _ in 0..2 {
            assert!(matches!(
                reader.read_packet().await,
                Ok(Packet::LinkStatistics(_))
            ));
        }
        assert_eq!(
            reader.read_packet().await,
            Err(CrsfStreamError::UnexpectedEof)
        );
    }
    let mut reader = AsyncCrsfReader::new(ChunkReader {
        input: &VALID_FRAME,
        chunk_size: 7,
        reads: Rc::new(Cell::new(0)),
        fail_on: Some(2),
    });
    assert_eq!(
        reader.read_packet().await,
        Err(CrsfStreamError::Io(embedded_io::ErrorKind::Other))
    );
    assert!(matches!(
        reader.read_packet().await,
        Ok(Packet::LinkStatistics(_))
    ));
}

#[futures_test::test]
async fn retained_frames_are_returned_before_reading_transport_again() {
    use common::recovery_stream;
    use common::ChunkReader;
    use std::cell::Cell;
    use std::rc::Rc;
    let bytes = recovery_stream();
    let reads = Rc::new(Cell::new(0));
    let mut reader = AsyncCrsfReader::new(ChunkReader {
        input: &bytes,
        chunk_size: 16,
        reads: reads.clone(),
        fail_on: None,
    });
    assert!(matches!(
        reader.read_packet().await,
        Err(CrsfStreamError::InvalidCrc { .. })
    ));
    assert_eq!(reads.get(), 2);
    assert_eq!(
        reader.read_packet().await,
        Err(CrsfStreamError::InvalidSync(30))
    );
    assert_eq!(
        reader.read_packet().await,
        Err(CrsfStreamError::InvalidSync(0xFF))
    );
    for _ in 0..2 {
        assert!(matches!(
            reader.read_packet().await,
            Ok(Packet::LinkStatistics(_))
        ));
        assert_eq!(reads.get(), 2);
    }
    // The rejected outer CRC is still a recovery byte; drain it before EOF.
    assert_eq!(
        reader.read_packet().await,
        Err(CrsfStreamError::InvalidSync(*bytes.last().unwrap()))
    );
    assert_eq!(reads.get(), 2);
    assert_eq!(
        reader.read_packet().await,
        Err(CrsfStreamError::UnexpectedEof)
    );
    assert_eq!(reads.get(), 3);
}
