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

fn push_bytes(parser: &mut CrsfParser, input: &[u8], events: &mut Vec<Event>) {
    for &byte in input {
        match parser.push_byte_raw(byte) {
            Ok(Some(packet)) => events.push(Ok(owned(packet))),
            Ok(None) => (),
            Err(error) => events.push(Err(error)),
        }
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
fn test_matches_byte_parser_across_splits_and_api_switches() {
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
    push_bytes(&mut baseline, &stream, &mut expected);
    push_bytes(&mut baseline, &max[31..], &mut expected);

    for split in 0..=stream.len() {
        for mode in 0..3 {
            let mut parser = CrsfParser::new();
            let mut events = Vec::new();
            if mode == 1 {
                push_bytes(&mut parser, &stream[..split], &mut events);
            } else {
                push_chunk(&mut parser, &stream[..split], &mut events);
            }
            push_chunk(&mut parser, &[], &mut events);
            if mode == 2 {
                push_bytes(&mut parser, &stream[split..], &mut events);
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
    // The failed candidate's body is scanned again, one byte at a time.
    for &byte in &bad_crc[1..bad_crc.len() - 1] {
        assert_eq!(
            parser.next_raw_packet(&mut cursor).unwrap().unwrap_err(),
            CrsfStreamError::InvalidSync(byte)
        );
    }
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
fn test_arbitrary_streams_match_byte_parser() {
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
        let mut baseline = CrsfParser::new();
        push_bytes(&mut baseline, &input, &mut expected);
        push_chunk(&mut baseline, &[], &mut expected);
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
fn test_typed_iteration_matches_bytes_and_raw_decoding_across_every_split() {
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
    for &byte in input.iter().chain(&valid[7..]) {
        match baseline.push_byte(byte) {
            Ok(Some(packet)) => expected.push(Ok(packet)),
            Ok(None) => (),
            Err(error) => expected.push(Err(error)),
        }
    }
    assert!(expected
        .iter()
        .any(|event| matches!(event, Err(CrsfStreamError::ParsingError(_)))));

    for split in 0..=input.len() {
        for mode in 0..3 {
            let mut parser = CrsfParser::new();
            let mut actual = Vec::new();
            match mode {
                0 => actual.extend(parser.iter_packets(&input[..split])),
                1 => {
                    for &byte in &input[..split] {
                        match parser.push_byte(byte) {
                            Ok(Some(packet)) => actual.push(Ok(packet)),
                            Ok(None) => (),
                            Err(error) => actual.push(Err(error)),
                        }
                    }
                }
                _ => {
                    let mut chunk = &input[..split];
                    while let Some(result) = parser.next_packet(&mut chunk) {
                        actual.push(result);
                    }
                }
            }
            actual.extend(parser.iter_packets(&[]));
            actual.extend(parser.iter_packets(&input[split..]));
            actual.extend(parser.iter_packets(&valid[7..]));
            assert_eq!(actual, expected, "split={split}, mode={mode}");
        }
    }

    for size in [1, 2, 3, 7, 63, 64] {
        let mut parser = CrsfParser::new();
        let mut actual = Vec::new();
        for chunk in input.chunks(size) {
            actual.extend(parser.iter_packets(chunk));
        }
        actual.extend(parser.iter_packets(&valid[7..]));
        assert_eq!(actual, expected, "chunk size={size}");
    }
}

#[test]
fn test_owned_packets_can_be_kept_while_iteration_continues() {
    let valid = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
    let input = [valid, valid].concat();
    let mut parser = CrsfParser::new();
    let mut packets = parser.iter_packets(&input);
    let first = packets.next().unwrap().unwrap();
    assert_eq!(packets.remaining(), valid);
    let second = packets.next().unwrap().unwrap();
    assert_eq!(first, second);
    assert!(packets.next().is_none());
    assert!(packets.remaining().is_empty());
}

/// An RC channels frame as EdgeTX sends it (sticks centred, AUX low). Its last
/// bytes `80 0f` read as a start byte (`TbsCorePnpPro`) and a valid length.
const EDGETX_RC: [u8; 26] = [
    0xC8, 0x18, 0x16, 0xE0, 0x6B, 0x1E, 0x83, 0xC0, 0xD7, 0x8A, 0x56, 0xB4, 0xA2, 0x15, 0xAD, 0x68,
    0x45, 0x2B, 0x5A, 0xD1, 0x8A, 0x56, 0x80, 0x0F, 0x7C, 0xA7,
];

fn frames_in(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, Ok(frame) if frame == &EDGETX_RC))
        .count()
}

fn events_by_every_api(stream: &[u8]) -> [(&'static str, Vec<Event>); 3] {
    let mut bytes = Vec::new();
    let mut parser = CrsfParser::new();
    push_bytes(&mut parser, stream, &mut bytes);
    push_chunk(&mut parser, &[], &mut bytes);

    let mut frame_chunks = Vec::new();
    let mut parser = CrsfParser::new();
    for chunk in stream.chunks(EDGETX_RC.len()) {
        push_chunk(&mut parser, chunk, &mut frame_chunks);
    }

    let mut one_chunk = Vec::new();
    push_chunk(&mut CrsfParser::new(), stream, &mut one_chunk);
    [
        ("bytes", bytes),
        ("frame-sized chunks", frame_chunks),
        ("one chunk", one_chunk),
    ]
}

#[test]
fn test_a_frame_cut_short_does_not_lock_out_the_frames_after_it() {
    // A link that drops mid-frame leaves a partial candidate. Skipping a failed
    // candidate's bytes made the parser take the `80 0f` near the end of every
    // following frame as a start, which swallowed the real start byte each time.
    for cut in 1..EDGETX_RC.len() {
        let mut stream = EDGETX_RC[..cut].to_vec();
        for _ in 0..8 {
            stream.extend_from_slice(&EDGETX_RC);
        }
        for (api, events) in events_by_every_api(&stream) {
            assert_eq!(frames_in(&events), 8, "cut={cut}, api={api}");
        }
    }
}

#[test]
fn test_a_stream_joined_mid_frame_finds_the_next_frame() {
    for offset in 1..EDGETX_RC.len() {
        let mut stream = EDGETX_RC[offset..].to_vec();
        for _ in 0..8 {
            stream.extend_from_slice(&EDGETX_RC);
        }
        for (api, events) in events_by_every_api(&stream) {
            assert_eq!(frames_in(&events), 8, "offset={offset}, api={api}");
        }
    }
}

#[test]
fn test_a_frame_inside_a_failed_candidate_is_found() {
    let inner = frame(14);
    let mut stream = vec![0xC8, 40]; // False start whose length covers `inner`.
    stream.extend_from_slice(&inner);
    stream.resize(42, 0x35); // Filler up to the false candidate's CRC position.
    stream.push(0x00); // Its CRC, wrong.

    let mut expected = Vec::new();
    let mut baseline = CrsfParser::new();
    push_bytes(&mut baseline, &stream, &mut expected);
    push_chunk(&mut baseline, &[], &mut expected);
    assert!(expected.contains(&Ok(inner.clone())), "{expected:?}");

    for size in 1..=stream.len() {
        let mut parser = CrsfParser::new();
        let mut events = Vec::new();
        for chunk in stream.chunks(size) {
            push_chunk(&mut parser, chunk, &mut events);
        }
        assert_eq!(events, expected, "chunk size={size}");
    }
}
