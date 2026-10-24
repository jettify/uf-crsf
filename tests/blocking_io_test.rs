#![cfg(feature = "embedded_io")]
#![cfg(test)]
extern crate std;

use uf_crsf::blocking_io::write_packet;
use uf_crsf::blocking_io::BlockingCrsfReader;
use uf_crsf::packets::LinkStatistics;
use uf_crsf::packets::Packet;
use uf_crsf::packets::PacketAddress;
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
    write_packet(&mut buffer, PacketAddress::FlightController, &packet).unwrap();
    buffer
}

#[test]
fn test_write_and_read_packet_blocking() {
    let packet_bytes = build_link_statistics_packet_bytes();

    // Mock reader
    let mut reader = &packet_bytes[..];

    let mut crsf_reader = BlockingCrsfReader::new(&mut reader);
    let result = crsf_reader.read_packet();

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
    let mut crsf_reader = BlockingCrsfReader::new(&mut reader);
    let result = crsf_reader.read_packet();
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[test]
fn test_read_packet_blocking_with_incomplete_data() {
    let packet_bytes = build_link_statistics_packet_bytes();
    let mut reader = &packet_bytes[..packet_bytes.len() - 1];
    let mut crsf_reader = BlockingCrsfReader::new(&mut reader);
    let result = crsf_reader.read_packet();
    assert!(matches!(result, Err(CrsfStreamError::UnexpectedEof)));
}

#[test]
fn test_read_packet_blocking_with_garbage() {
    let garbage = [0x01, 0x02, 0x03];
    let mut reader = &garbage[..];
    let mut crsf_reader = BlockingCrsfReader::new(&mut reader);
    let result = crsf_reader.read_packet();
    // We expect an InvalidSync error because the first byte is not a valid sync byte.
    assert!(matches!(result, Err(CrsfStreamError::InvalidSync(_))));
}

mod common;

#[test]
fn buffered_results_resume_without_transport_reads() {
    use common::{ChunkReader, VALID_FRAME};
    use std::{cell::Cell, rc::Rc};
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
    let mut reader = BlockingCrsfReader::new(ChunkReader {
        input: &bytes,
        chunk_size: 64,
        reads: reads.clone(),
        fail_on: None,
    });
    assert_eq!(
        reader.read_packet(),
        Err(CrsfStreamError::InvalidSync(0xFF))
    );
    assert!(matches!(
        reader.read_packet(),
        Ok(Packet::LinkStatistics(_))
    ));
    assert!(matches!(
        reader.read_packet(),
        Err(CrsfStreamError::InvalidCrc { .. })
    ));
    assert!(matches!(
        reader.read_packet(),
        Err(CrsfStreamError::ParsingError(_))
    ));
    assert!(matches!(
        reader.read_packet(),
        Ok(Packet::LinkStatistics(_))
    ));
    assert_eq!(reads.get(), 1);
    assert_eq!(reader.read_packet(), Err(CrsfStreamError::UnexpectedEof));
    assert_eq!(reads.get(), 2);
}

#[test]
fn split_reads_and_transport_error_preserve_partial_frames() {
    use common::{ChunkReader, VALID_FRAME};
    use std::{cell::Cell, rc::Rc};
    let bytes = [VALID_FRAME, VALID_FRAME].concat();
    for chunk_size in [1, 2, 7, 64] {
        let mut reader = BlockingCrsfReader::new(ChunkReader {
            input: &bytes,
            chunk_size,
            reads: Rc::new(Cell::new(0)),
            fail_on: None,
        });
        for _ in 0..2 {
            assert!(matches!(
                reader.read_packet(),
                Ok(Packet::LinkStatistics(_))
            ));
        }
        assert_eq!(reader.read_packet(), Err(CrsfStreamError::UnexpectedEof));
    }
    let mut reader = BlockingCrsfReader::new(ChunkReader {
        input: &VALID_FRAME,
        chunk_size: 7,
        reads: Rc::new(Cell::new(0)),
        fail_on: Some(2),
    });
    assert_eq!(
        reader.read_packet(),
        Err(CrsfStreamError::Io(embedded_io::ErrorKind::Other))
    );
    assert!(matches!(
        reader.read_packet(),
        Ok(Packet::LinkStatistics(_))
    ));
}
