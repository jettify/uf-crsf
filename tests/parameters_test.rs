use uf_crsf::packets::*;
use uf_crsf::CrsfParser;
use uf_crsf::CrsfParsingError;

fn check_wire<T: CrsfPacket + core::fmt::Debug + PartialEq>(
    packet: T,
    expected: &[u8],
    wrap: fn(T) -> Packet,
) {
    let mut payload = [0xcc; 60];
    let len = packet.to_bytes(&mut payload).unwrap();
    assert_eq!(&payload[..len], expected);
    assert_eq!(T::from_bytes(expected).unwrap(), packet);

    // Check that framing includes routing bytes exactly once and dispatches
    // through the public parser, including maximum-size 64-byte frames.
    let mut frame = [0; 64];
    let frame_len =
        write_packet_to_buffer(&mut frame, PacketAddress::FlightController, &packet).unwrap();
    assert_eq!(frame_len, expected.len() + 4);
    assert_eq!(
        &frame[..3],
        &[0xc8, (expected.len() + 2) as u8, T::PACKET_TYPE as u8]
    );
    assert_eq!(&frame[3..frame_len - 1], expected);
    let crc = crc::Crc::<u8>::new(&crc::CRC_8_DVB_S2).checksum(&frame[2..frame_len - 1]);
    assert_eq!(frame[frame_len - 1], crc);
    let parsed = CrsfParser::new()
        .iter_packets(&mut &frame[..frame_len])
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(parsed, wrap(packet));

    // A short output buffer must fail without partially writing a frame payload.
    let mut short = vec![0xcc; len - 1];
    let decoded = T::from_bytes(expected).unwrap();
    assert_eq!(
        decoded.to_bytes(&mut short),
        Err(CrsfParsingError::BufferOverflow)
    );
    assert!(short.iter().all(|&byte| byte == 0xcc));
    for size in 0..T::MIN_PAYLOAD_SIZE {
        assert_eq!(
            T::from_bytes(&expected[..size]),
            Err(CrsfParsingError::InvalidPayloadLength)
        );
    }
}

#[test]
fn read_requests_include_root_and_chunk_indexes() {
    for (parameter, chunk) in [(0, 0), (2, 1), (255, 255)] {
        check_wire(
            ParameterRead::new(0xee, 0xea, parameter, chunk).unwrap(),
            &[0xee, 0xea, parameter, chunk],
            Packet::ParameterRead,
        );
    }
}

#[test]
fn entry_preserves_root_definition_and_arbitrary_chunk_boundaries() {
    // Parent zero, FOLDER type, name ROOT, then child IDs and terminator.
    let definition = [0, 11, b'R', b'O', b'O', b'T', 0, 1, 2, 0xff];
    let mut reassembled = Vec::new();
    for (chunk, remaining) in [(&definition[..4], 1), (&definition[4..], 0)] {
        let packet = ParameterSettingsEntry::new(0xea, 0xee, 0, remaining, chunk).unwrap();
        assert_eq!(packet.data(), chunk);
        let mut expected = vec![0xea, 0xee, 0, remaining];
        expected.extend_from_slice(chunk);
        check_wire(packet, &expected, Packet::ParameterSettingsEntry);
        let decoded = ParameterSettingsEntry::from_bytes(&expected).unwrap();
        reassembled.extend_from_slice(decoded.data());
    }
    assert_eq!(reassembled, definition);
}

#[test]
fn writes_and_value_acknowledgements_preserve_raw_values() {
    // Selection, signed integer backing a FLOAT, NUL-terminated string, command.
    for value in [&[3][..], &(-12345i32).to_be_bytes(), b"Pilot\0", &[1]] {
        for (dst, src) in [(0xee, 0xea), (0xea, 0xee)] {
            let packet = ParameterWrite::new(dst, src, 7, value).unwrap();
            assert_eq!(packet.data(), value);
            let mut expected = vec![dst, src, 7];
            expected.extend_from_slice(value);
            check_wire(packet, &expected, Packet::ParameterWrite);
        }
    }
}

#[test]
fn raw_data_lengths_cover_empty_through_maximum_frame() {
    for size in 0..=56 {
        let mut expected = vec![0xea, 0xee, 255, 255];
        expected.extend_from_slice(&[0xa5; 56][..size]);
        check_wire(
            ParameterSettingsEntry::new(0xea, 0xee, 255, 255, &expected[4..]).unwrap(),
            &expected,
            Packet::ParameterSettingsEntry,
        );
    }
    for size in 0..=57 {
        let mut expected = vec![0xee, 0xea, 255];
        expected.extend_from_slice(&[0xa5; 57][..size]);
        check_wire(
            ParameterWrite::new(0xee, 0xea, 255, &expected[3..]).unwrap(),
            &expected,
            Packet::ParameterWrite,
        );
    }
}

#[test]
fn raw_payloads_reject_data_exceeding_frame_capacity() {
    assert_eq!(
        ParameterSettingsEntry::new(0xea, 0xee, 1, 0, &[0; 57]),
        Err(CrsfParsingError::InvalidPayloadLength),
    );
    assert_eq!(
        ParameterWrite::new(0xee, 0xea, 1, &[0; 58]),
        Err(CrsfParsingError::InvalidPayloadLength),
    );
    assert_eq!(
        ParameterSettingsEntry::from_bytes(&[0; 61]),
        Err(CrsfParsingError::InvalidPayloadLength)
    );
    assert_eq!(
        ParameterWrite::from_bytes(&[0; 61]),
        Err(CrsfParsingError::InvalidPayloadLength)
    );
}
