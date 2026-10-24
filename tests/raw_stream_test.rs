use uf_crsf::CrsfParser;
use uf_crsf::CrsfStreamError;
use uf_crsf::RawCrsfPacket;

type Event = Result<Vec<u8>, CrsfStreamError>;

fn frame(size: usize) -> Vec<u8> {
    let mut bytes = vec![0xC8, (size - 2) as u8, 0xFE];
    bytes.resize(size - 1, 0x35);
    bytes.push(crc::Crc::<u8>::new(&crc::CRC_8_DVB_S2).checksum(&bytes[2..]));
    bytes
}

fn owned(packet: RawCrsfPacket<'_>) -> Vec<u8> {
    let mut bytes = vec![
        packet.dst_addr(),
        (packet.len() - 2) as u8,
        packet.raw_packet_type(),
    ];
    bytes.extend_from_slice(packet.payload());
    bytes.push(packet.crc());
    assert_eq!(packet.as_slice(), bytes.as_slice());
    assert_eq!(packet.as_ref(), bytes.as_slice());
    packet.as_slice().to_vec()
}

fn push_single_byte_slices(parser: &mut CrsfParser, input: &[u8], events: &mut Vec<Event>) {
    for byte in input {
        push_chunk(parser, core::slice::from_ref(byte), events);
    }
}

fn push_chunk(parser: &mut CrsfParser, mut input: &[u8], events: &mut Vec<Event>) {
    while let Some(result) = parser.next_raw_packet(&mut input) {
        events.push(result.map(owned));
    }
    assert!(input.is_empty());
    assert!(parser.next_raw_packet(&mut input).is_none());
}

#[test]
fn test_matches_single_byte_slices_across_splits_and_chunk_sizes() {
    let min = frame(4);
    let max = frame(64);
    let mut bad_crc = frame(9);
    *bad_crc.last_mut().unwrap() = 0xC8;
    assert_ne!(bad_crc.last(), frame(9).last());
    let mut stream = vec![0xFF, 0x01, 0xC8, 0xC8]; // Length reused as sync.
    stream.extend_from_slice(&min[1..]);
    stream.extend_from_slice(&max);
    stream.extend_from_slice(&[0xC8, 0, 0xC8, 63, 0xFF]);
    stream.extend_from_slice(&bad_crc);
    stream.extend_from_slice(&min[1..]); // CRC reused as sync.
    stream.extend_from_slice(&frame(14));
    stream.extend_from_slice(&max[..31]); // Leave a partial candidate.

    let mut expected = Vec::new();
    let mut baseline = CrsfParser::new();
    push_single_byte_slices(&mut baseline, &stream, &mut expected);
    push_single_byte_slices(&mut baseline, &max[31..], &mut expected);

    for split in 0..=stream.len() {
        for mode in 0..3 {
            let mut parser = CrsfParser::new();
            let mut events = Vec::new();
            if mode == 1 {
                push_single_byte_slices(&mut parser, &stream[..split], &mut events);
            } else {
                push_chunk(&mut parser, &stream[..split], &mut events);
            }
            push_chunk(&mut parser, &[], &mut events);
            if mode == 2 {
                push_single_byte_slices(&mut parser, &stream[split..], &mut events);
            } else {
                push_chunk(&mut parser, &stream[split..], &mut events);
            }
            push_chunk(&mut parser, &max[31..], &mut events);
            assert_eq!(events, expected, "split={split}, mode={mode}");
        }
    }

    for size in 1..=stream.len() {
        let mut parser = CrsfParser::new();
        let mut events = Vec::new();
        for chunk in stream.chunks(size) {
            push_chunk(&mut parser, chunk, &mut events);
        }
        push_chunk(&mut parser, &max[31..], &mut events);
        assert_eq!(events, expected, "chunk size={size}");
    }
}

#[test]
fn test_contiguous_frames_borrow_input_and_remaining_can_be_resumed() {
    let first = frame(14);
    let second = frame(64);
    let buf = [first.clone(), second.clone()].concat();
    let mut parser = CrsfParser::new();
    let mut input = buf.as_slice();
    let packet = parser.next_raw_packet(&mut input).unwrap().unwrap();
    assert_eq!(packet.as_slice().as_ptr(), buf.as_ptr());
    assert_eq!(packet.payload().as_ptr(), buf[3..].as_ptr());
    assert_eq!(owned(packet), first);
    assert_eq!(input, second);

    let suffix = input;
    let packet = parser.next_raw_packet(&mut input).unwrap().unwrap();
    assert_eq!(packet.as_slice().as_ptr(), suffix.as_ptr());
    assert_eq!(packet.payload().as_ptr(), suffix[3..].as_ptr());
    assert_eq!(owned(packet), second);
    assert!(parser.next_raw_packet(&mut input).is_none());
    assert!(input.is_empty());
}

#[test]
fn test_resynchronized_frames_stay_zero_copy() {
    let valid = frame(14);
    let mut bad_crc = frame(9);
    *bad_crc.last_mut().unwrap() = 0xC8;

    // 1. False sync byte (0xC8) followed immediately by `valid` (whose sync is 0xC8).
    // 2. `bad_crc` whose corrupt CRC byte (0xC8) is the sync byte of the second `valid` frame.
    let mut input = vec![0xC8];
    let first_offset = input.len();
    input.extend_from_slice(&valid);
    let second_offset = input.len() + bad_crc.len() - 1;
    input.extend_from_slice(&bad_crc);
    input.extend_from_slice(&valid[1..]);

    let mut parser = CrsfParser::new();
    let mut cursor = input.as_slice();

    assert!(matches!(
        parser.next_raw_packet(&mut cursor),
        Some(Err(CrsfStreamError::InvalidPacketLength(0xC8)))
    ));
    let first_pkt = parser.next_raw_packet(&mut cursor).unwrap().unwrap();
    assert_eq!(
        first_pkt.as_slice().as_ptr(),
        input[first_offset..].as_ptr()
    );
    assert_eq!(first_pkt.as_slice(), valid.as_slice());

    assert!(matches!(
        parser.next_raw_packet(&mut cursor),
        Some(Err(CrsfStreamError::InvalidCrc { .. }))
    ));
    let second_pkt = parser.next_raw_packet(&mut cursor).unwrap().unwrap();
    assert_eq!(
        second_pkt.as_slice().as_ptr(),
        input[second_offset..].as_ptr()
    );
    assert_eq!(second_pkt.as_slice(), valid.as_slice());
    assert!(parser.next_raw_packet(&mut cursor).is_none());
    assert!(cursor.is_empty());
}

#[test]
fn test_partial_chunk_consumes_input_and_preserves_state() {
    let input = frame(14);
    let mut parser = CrsfParser::new();
    let mut first_half = &input[..7];
    assert!(parser.next_raw_packet(&mut first_half).is_none());
    // Consumed bytes belong to parser storage, not the unread suffix.
    assert!(first_half.is_empty());
    assert!(parser.next_raw_packet(&mut first_half).is_none());

    let mut second_half = &input[7..];
    assert_eq!(
        owned(parser.next_raw_packet(&mut second_half).unwrap().unwrap()),
        input
    );
    assert!(second_half.is_empty());
    assert!(parser.next_raw_packet(&mut second_half).is_none());
}

#[test]
fn test_stopping_early_does_not_consume_unread_partial_frame() {
    let first = frame(14);
    let second = frame(64);
    let buf = [first.clone(), second[..7].to_vec()].concat();
    let mut parser = CrsfParser::new();
    let mut input = buf.as_slice();
    assert_eq!(
        owned(parser.next_raw_packet(&mut input).unwrap().unwrap()),
        first
    );
    assert_eq!(input, &second[..7]);

    // Feed the entire second packet: stopping after the first packet did not
    // eagerly buffer the trailing partial prefix.
    let mut full_second = second.as_slice();
    assert_eq!(
        owned(parser.next_raw_packet(&mut full_second).unwrap().unwrap()),
        second
    );
    assert!(parser.next_raw_packet(&mut full_second).is_none());
    assert!(full_second.is_empty());
}

#[test]
fn test_reset_discards_spillover_and_partial_chunks_consume_their_input() {
    let input = frame(64);
    let mut parser = CrsfParser::new();
    let mut events = Vec::new();
    push_chunk(&mut parser, &input[..32], &mut events);
    assert!(events.is_empty());
    parser.reset();
    push_chunk(&mut parser, &input, &mut events);
    assert_eq!(events, vec![Ok(input)]);
}

#[test]
fn test_arbitrary_streams_match_single_byte_slices() {
    let mut seed = 0x12345678u32;
    for _ in 0..32 {
        let mut input = Vec::new();
        for _ in 0..256 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            input.push(seed as u8);
        }
        input.extend_from_slice(&frame(14));
        let mut expected = Vec::new();
        push_single_byte_slices(&mut CrsfParser::new(), &input, &mut expected);
        for size in [1, 2, 3, 7, 63, 64, input.len()] {
            let mut parser = CrsfParser::new();
            let mut events = Vec::new();
            for chunk in input.chunks(size) {
                push_chunk(&mut parser, chunk, &mut events);
            }
            assert_eq!(events, expected, "chunk size={size}");
        }
    }
}

#[test]
fn test_typed_iteration_matches_single_byte_slices_and_cursor_across_every_split() {
    let valid = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
    let mut bad_crc = valid;
    bad_crc[13] ^= 1;
    let mut invalid_payload = frame(4);
    invalid_payload[2] = 0x14;
    invalid_payload[3] = crc::Crc::<u8>::new(&crc::CRC_8_DVB_S2).checksum(&invalid_payload[2..3]);
    let mut input = vec![0xFF, 0xC8, 0xC8];
    input.extend_from_slice(&valid[1..]);
    input.extend_from_slice(&bad_crc);
    input.extend_from_slice(&invalid_payload);
    input.extend_from_slice(&frame(64)); // Valid framing, unsupported packet type.
    input.extend_from_slice(&valid);
    input.extend_from_slice(&valid[..7]); // Persist a trailing partial frame.

    let mut baseline = CrsfParser::new();
    let mut expected = Vec::new();
    for byte in input.iter().chain(&valid[7..]) {
        expected.extend(baseline.iter_packets(&mut core::slice::from_ref(byte)));
    }
    assert!(expected
        .iter()
        .any(|event| matches!(event, Err(CrsfStreamError::ParsingError(_)))));

    for split in 0..=input.len() {
        for mode in 0..3 {
            let mut parser = CrsfParser::new();
            let mut actual = Vec::new();
            match mode {
                0 => actual.extend(parser.iter_packets(&mut &input[..split])),
                1 => {
                    for byte in &input[..split] {
                        actual.extend(parser.iter_packets(&mut core::slice::from_ref(byte)));
                    }
                }
                _ => {
                    let mut chunk = &input[..split];
                    while let Some(result) = parser.next_packet(&mut chunk) {
                        actual.push(result);
                    }
                }
            }
            actual.extend(parser.iter_packets(&mut &(&[])[..]));
            actual.extend(parser.iter_packets(&mut &input[split..]));
            actual.extend(parser.iter_packets(&mut &valid[7..]));
            assert_eq!(actual, expected, "split={split}, mode={mode}");
        }
    }

    for size in [1, 2, 3, 7, 63, 64] {
        let mut parser = CrsfParser::new();
        let mut actual = Vec::new();
        for chunk in input.chunks(size) {
            actual.extend(parser.iter_packets(&mut &chunk[..]));
        }
        actual.extend(parser.iter_packets(&mut &valid[7..]));
        assert_eq!(actual, expected, "chunk size={size}");
    }
}

#[test]
fn test_owned_packets_can_be_kept_while_iteration_continues() {
    let valid = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
    let input = [valid, valid].concat();
    let mut parser = CrsfParser::new();
    let mut cursor = input.as_slice();
    let mut packets = parser.iter_packets(&mut cursor);
    let first = packets.next().unwrap().unwrap();
    assert_eq!(packets.remaining(), valid);
    let second = packets.next().unwrap().unwrap();
    assert_eq!(first, second);
    assert!(packets.next().is_none());
    assert!(packets.remaining().is_empty());
}

#[test]
fn typed_iterator_early_exit_preserves_cursor() {
    let valid = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
    let bytes = [valid.as_slice(), &valid[..7]].concat();
    for mode in 0..2 {
        let mut parser = CrsfParser::new();
        let mut input = bytes.as_slice();
        if mode == 0 {
            for result in parser.iter_packets(&mut input) {
                if matches!(result.unwrap(), uf_crsf::Packet::LinkStatistics(_)) {
                    break;
                }
            }
        } else {
            assert_eq!(parser.iter_packets(&mut input).take(1).count(), 1);
        }
        assert_eq!(input, &valid[..7]);
        // No eager buffering on drop: a full new frame still parses normally.
        assert!(parser.next_packet(&mut valid.as_slice()).unwrap().is_ok());
    }

    fn until_error(parser: &mut CrsfParser, input: &mut &[u8]) -> Result<(), CrsfStreamError> {
        for result in parser.iter_packets(input) {
            result?;
        }
        Ok(())
    }
    let bytes = [&[0xFF][..], &valid].concat();
    let mut input = bytes.as_slice();
    let mut parser = CrsfParser::new();
    assert_eq!(
        until_error(&mut parser, &mut input),
        Err(CrsfStreamError::InvalidSync(0xFF))
    );
    assert_eq!(input, valid);
    assert!(parser.iter_packets(&mut input).next().unwrap().is_ok());
    assert!(input.is_empty());
}

#[test]
fn raw_errors_have_explicit_expected_order_across_chunks() {
    let mut bad_crc = frame(4);
    bad_crc[3] ^= 1;
    let valid = frame(4);
    let bytes = [&[0xFF, 0xC8, 0][..], &bad_crc, &valid].concat();
    let expected = vec![
        Err(CrsfStreamError::InvalidSync(0xFF)),
        Err(CrsfStreamError::InvalidPacketLength(0)),
        // Broadcast address 0 is reused as sync before the following 0xC8.
        Err(CrsfStreamError::InvalidPacketLength(0xC8)),
        Err(CrsfStreamError::InvalidCrc {
            calculated_crc: valid[3],
            packet_crc: bad_crc[3],
        }),
        Ok(valid),
    ];
    for size in 1..=bytes.len() {
        let mut parser = CrsfParser::new();
        let mut actual = Vec::new();
        for chunk in bytes.chunks(size) {
            push_chunk(&mut parser, chunk, &mut actual);
        }
        assert_eq!(actual, expected, "chunk size={size}");
    }
}
