use uf_crsf::packets::{
    ArduPilotLegacy, ArduPilotPassthrough, Packet, PacketAddress, PacketType,
    PassthroughStatusText, PassthroughTelemetryPacket,
};
use uf_crsf::{write_packet_to_buffer, CrsfParser};

#[test]
fn independent_wire_fixtures() {
    // CRCs computed independently using polynomial 0xD5, initial value zero.
    let frames: [&[u8]; 6] = [
        &[
            0xc8, 0x09, 0x80, 0xf0, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0x0b,
        ],
        &[
            0xc8, 0x09, 0x7f, 0xf0, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0x94,
        ],
        &[
            0xc8, 0x10, 0x80, 0xf2, 0x02, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0xcd, 0xab, 0xef,
            0xcd, 0xab, 0x89, 0xe6,
        ],
        &[
            0xc8, 0x10, 0x7f, 0xf2, 0x02, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0xcd, 0xab, 0xef,
            0xcd, 0xab, 0x89, 0x89,
        ],
        &[0xc8, 0x08, 0x80, 0xf1, 0x04, 0x41, 0x42, 0x43, 0x00, 0x88],
        &[0xc8, 0x08, 0x7f, 0xf1, 0x04, 0x41, 0x42, 0x43, 0x00, 0xc5],
    ];
    let first = PassthroughTelemetryPacket {
        appid: 0x1234,
        data: 0x12345678,
    };
    let second = PassthroughTelemetryPacket {
        appid: 0xabcd,
        data: 0x89abcdef,
    };
    let packets = [
        ArduPilotPassthrough::Single(first),
        ArduPilotPassthrough::multi(&[first, second]).unwrap(),
        ArduPilotPassthrough::StatusText(PassthroughStatusText::new(4, b"ABC").unwrap()),
    ];
    assert_eq!(PacketType::try_from(0x7f), Ok(PacketType::ArdupilotLegacy));
    assert_eq!(
        PacketType::try_from(0x80),
        Ok(PacketType::ArdupilotResponse)
    );
    assert!(!PacketType::ArdupilotLegacy.is_extended());
    assert!(!PacketType::ArdupilotResponse.is_extended());
    for (index, frame) in frames.iter().enumerate() {
        let packet = packets[index / 2].clone();
        let mut buffer = [0xaa; 64];
        let (len, expected) = if index % 2 == 0 {
            (
                write_packet_to_buffer(&mut buffer, PacketAddress::FlightController, &packet)
                    .unwrap(),
                Packet::ArduPilotPassthrough(packet),
            )
        } else {
            let packet = ArduPilotLegacy(packet);
            (
                write_packet_to_buffer(&mut buffer, PacketAddress::FlightController, &packet)
                    .unwrap(),
                Packet::ArduPilotLegacy(packet),
            )
        };
        assert_eq!(&buffer[..len], *frame);
        assert_eq!(buffer[len], 0xaa);
        for split in 0..=frame.len() {
            let mut parser = CrsfParser::new();
            let mut parsed: Vec<_> = parser.iter_packets(&frame[..split]).collect();
            parsed.extend(parser.iter_packets(&frame[split..]));
            assert_eq!(parsed, vec![Ok(expected.clone())]);
        }
    }
}

#[test]
fn maximum_payloads_and_broadcast_address() {
    let item = PassthroughTelemetryPacket {
        appid: 0xffff,
        data: 0xffffffff,
    };
    let packets = [
        ArduPilotPassthrough::multi(&[item; 9]).unwrap(),
        ArduPilotPassthrough::multi(&[]).unwrap(),
        ArduPilotPassthrough::StatusText(PassthroughStatusText::new(255, &[0xff; 50]).unwrap()),
        ArduPilotPassthrough::StatusText(PassthroughStatusText::new(0, b"").unwrap()),
    ];
    for packet in packets {
        for legacy in [false, true] {
            let mut buffer = [0; 64];
            let (len, expected) = if legacy {
                let packet = ArduPilotLegacy(packet.clone());
                (
                    write_packet_to_buffer(&mut buffer, PacketAddress::Broadcast, &packet).unwrap(),
                    Packet::ArduPilotLegacy(packet),
                )
            } else {
                (
                    write_packet_to_buffer(&mut buffer, PacketAddress::Broadcast, &packet).unwrap(),
                    Packet::ArduPilotPassthrough(packet.clone()),
                )
            };
            assert_eq!(buffer[0], 0);
            assert!(len <= 60);
            assert_eq!(
                CrsfParser::new()
                    .iter_packets(&buffer[..len])
                    .collect::<Vec<_>>(),
                vec![Ok(expected)]
            );
        }
    }
}
