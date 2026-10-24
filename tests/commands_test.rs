use uf_crsf::packets::BluetoothCommand;
use uf_crsf::packets::CommandAck;
use uf_crsf::packets::CommandPayload;
use uf_crsf::packets::CommandString;
use uf_crsf::packets::CrossfireCommand;
use uf_crsf::packets::CrsfPacket;
use uf_crsf::packets::DirectCommands;
use uf_crsf::packets::FcCommand;
use uf_crsf::packets::FlowControlCommand;
use uf_crsf::packets::GeneralCommand;
use uf_crsf::packets::LedColor;
use uf_crsf::packets::LedCommand;
use uf_crsf::packets::OsdCommand;
use uf_crsf::packets::RcOverWifiCommand;
use uf_crsf::packets::ScreenAdditionalData;
use uf_crsf::packets::ScreenCommand;
use uf_crsf::packets::ScreenPopup;
use uf_crsf::packets::ScreenPopupFields;
use uf_crsf::packets::ScreenSelection;
use uf_crsf::packets::VtxCommand;
use uf_crsf::write_packet_to_buffer;
use uf_crsf::CrsfParser;
use uf_crsf::CrsfParsingError;
use uf_crsf::Packet;
use uf_crsf::PacketAddress;

// Independent reference CRCs, rather than the codec's CRC implementation.
fn crc(data: &[u8], polynomial: u8) -> u8 {
    let mut value = 0;
    for byte in data {
        value ^= byte;
        for _ in 0..8 {
            value = (value << 1) ^ if value & 0x80 != 0 { polynomial } else { 0 };
        }
    }
    value
}

fn wire(command_id: u8, body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x32, 0xEE, 0xEA, command_id];
    bytes.extend_from_slice(body);
    bytes.push(crc(&bytes, 0xBA));
    bytes.remove(0);
    bytes
}

fn check_fixture(command_id: u8, body: &[u8], payload: CommandPayload) {
    let expected = DirectCommands {
        dst_addr: 0xEE,
        src_addr: 0xEA,
        payload,
    };
    let bytes = wire(command_id, body);
    assert_eq!(DirectCommands::from_bytes(&bytes), Ok(expected.clone()));
    for size in 0..=bytes.len() {
        let mut buffer = vec![0; size];
        let result = expected.to_bytes(&mut buffer);
        if size < bytes.len() {
            assert_eq!(result, Err(CrsfParsingError::BufferOverflow));
        } else {
            assert_eq!(result, Ok(bytes.len()));
            assert_eq!(buffer, bytes);
        }
    }
    let mut frame = vec![0xC8, (bytes.len() + 2) as u8, 0x32];
    frame.extend_from_slice(&bytes);
    frame.push(crc(&frame[2..], 0xD5));
    let mut buffer = [0; 64];
    let len =
        write_packet_to_buffer(&mut buffer, PacketAddress::FlightController, &expected).unwrap();
    assert_eq!(&buffer[..len], frame);
    assert_eq!(
        CrsfParser::new()
            .iter_packets(&mut &frame[..])
            .collect::<Vec<_>>(),
        vec![Ok(Packet::Commands(expected))]
    );
}

#[test]
fn fixed_command_fixtures_and_extensions() {
    let start = LedColor {
        hue: 257,
        saturation: 65,
        value: 42,
    };
    let stop = LedColor {
        hue: 359,
        saturation: 100,
        value: 100,
    };
    let fixtures: Vec<(u8, &[u8], CommandPayload)> = vec![
        (3, &[1], CommandPayload::Bluetooth(BluetoothCommand::Reset)),
        (
            3,
            &[2, 0],
            CommandPayload::Bluetooth(BluetoothCommand::Enable(false)),
        ),
        (
            3,
            &[2, 1],
            CommandPayload::Bluetooth(BluetoothCommand::Enable(true)),
        ),
        (
            3,
            &[0x64],
            CommandPayload::Bluetooth(BluetoothCommand::Echo),
        ),
        (9, &[1], CommandPayload::Led(LedCommand::SetDefault)),
        (
            9,
            &[2, 0x80, 0xC1, 42],
            CommandPayload::Led(LedCommand::OverrideColor(start)),
        ),
        (
            9,
            &[3, 0x12, 0x34, 0x80, 0xC1, 42, 0xB3, 0xE4, 100],
            CommandPayload::Led(LedCommand::OverridePulse {
                duration_ms: 0x1234,
                start,
                stop,
            }),
        ),
        (
            9,
            &[4, 0x12, 0x34, 0x80, 0xC1, 42, 0xB3, 0xE4, 100],
            CommandPayload::Led(LedCommand::OverrideBlink {
                interval_ms: 0x1234,
                start,
                stop,
            }),
        ),
        (
            9,
            &[5, 0x12, 0x34, 0x80, 0xC1, 42],
            CommandPayload::Led(LedCommand::OverrideShift {
                interval_ms: 0x1234,
                color: start,
            }),
        ),
        (
            0x0A,
            &[0x70, 2, 1, 2, 3, 4],
            CommandPayload::General(GeneralCommand::SpeedProposal {
                port_id: 2,
                proposed_baudrate: 0x01020304,
            }),
        ),
        (
            0x0A,
            &[0x71, 2, 0],
            CommandPayload::General(GeneralCommand::SpeedProposalResponse {
                port_id: 2,
                accepted: false,
            }),
        ),
        (
            0x0A,
            &[0x71, 2, 1],
            CommandPayload::General(GeneralCommand::SpeedProposalResponse {
                port_id: 2,
                accepted: true,
            }),
        ),
        (
            0x10,
            &[0x0A],
            CommandPayload::Crossfire(CrossfireCommand::EnableRxTelemetry),
        ),
        (
            0x10,
            &[0x0B],
            CommandPayload::Crossfire(CrossfireCommand::DisableRxTelemetry),
        ),
        (
            0x10,
            &[0x0C],
            CommandPayload::Crossfire(CrossfireCommand::FactoryReset),
        ),
        (
            0x10,
            &[0x0D],
            CommandPayload::Crossfire(CrossfireCommand::ResetToFactoryFirmware),
        ),
        (
            0x13,
            &[1],
            CommandPayload::RcOverWifi(RcOverWifiCommand::Enable),
        ),
        (
            0x13,
            &[2],
            CommandPayload::RcOverWifi(RcOverWifiCommand::Disable),
        ),
        (
            0x22,
            &[2, 42, 0],
            CommandPayload::Screen(ScreenCommand::SelectionReturnValue {
                value: 42,
                response: false,
            }),
        ),
        (
            0x22,
            &[2, 42, 1],
            CommandPayload::Screen(ScreenCommand::SelectionReturnValue {
                value: 42,
                response: true,
            }),
        ),
    ];
    for (id, body, command) in fixtures {
        check_fixture(id, body, command.clone());
        for size in 0..body.len() {
            assert_eq!(
                DirectCommands::from_bytes(&wire(id, &body[..size])),
                Err(CrsfParsingError::InvalidPayloadLength)
            );
        }
        let mut extended = body.to_vec();
        extended.extend_from_slice(&[0xA5, 0xFF]);
        assert_eq!(
            DirectCommands::from_bytes(&wire(id, &extended))
                .unwrap()
                .payload,
            command
        );
    }
}

#[test]
fn wifi_spec_examples() {
    for (frame, command) in [
        (
            [0xC8, 7, 0x32, 0xEE, 0x13, 0x13, 1, 0x66, 6],
            RcOverWifiCommand::Enable,
        ),
        (
            [0xC8, 7, 0x32, 0xEE, 0x13, 0x13, 2, 0x12, 0x8E],
            RcOverWifiCommand::Disable,
        ),
    ] {
        let packet = DirectCommands {
            dst_addr: 0xEE,
            src_addr: 0x13,
            payload: CommandPayload::RcOverWifi(command),
        };
        let mut buffer = [0; 64];
        let len =
            write_packet_to_buffer(&mut buffer, PacketAddress::FlightController, &packet).unwrap();
        assert_eq!(&buffer[..len], frame);
        assert_eq!(
            CrsfParser::new()
                .iter_packets(&mut &frame[..])
                .collect::<Vec<_>>(),
            vec![Ok(Packet::Commands(packet))]
        );
    }
}

fn string(bytes: &[u8]) -> CommandString {
    CommandString::new(bytes).unwrap()
}

fn screen(fields: ScreenPopupFields) -> CommandPayload {
    let popup = ScreenPopup::new(&fields).unwrap();
    assert_eq!(popup.fields(), Ok(fields));
    CommandPayload::Screen(ScreenCommand::PopupMessageStart(popup))
}

fn popup() -> ScreenPopupFields {
    ScreenPopupFields {
        header: string(b"H"),
        info: string(b"I"),
        max_timeout_interval: 5,
        close_button_option: true,
        additional_data: None,
        possible_values: None,
    }
}

#[test]
fn screen_optional_field_fixtures() {
    let mut popup = popup();
    let required = [1, b'H', 0, b'I', 0, 5, 1];
    check_fixture(0x22, &required, screen(popup.clone()));
    for size in 0..required.len() {
        assert!(DirectCommands::from_bytes(&wire(0x22, &required[..size])).is_err());
    }
    popup.additional_data = Some(ScreenAdditionalData::Empty);
    let empty_data = [1, b'H', 0, b'I', 0, 5, 1, 0];
    check_fixture(0x22, &empty_data, screen(popup.clone()));
    popup.possible_values = Some(string(b""));
    check_fixture(
        0x22,
        &[1, b'H', 0, b'I', 0, 5, 1, 0, 0],
        screen(popup.clone()),
    );
    popup.possible_values = Some(string(b"Yes;No"));
    check_fixture(0x22, b"\x01H\0I\0\x05\x01\0Yes;No\0", screen(popup.clone()));
    popup.additional_data = Some(ScreenAdditionalData::Selection(ScreenSelection {
        selection_text: string(b"P"),
        value: 50,
        min_value: 0,
        max_value: 100,
        default_value: 25,
        unit: string(b"%"),
    }));
    popup.possible_values = None;
    let selection = b"\x01H\0I\0\x05\x01P\0\x32\0\x64\x19%\0";
    check_fixture(0x22, selection, screen(popup.clone()));
    // Once a nonempty selection starts, its remaining fields are mandatory.
    for size in required.len() + 1..selection.len() {
        assert!(DirectCommands::from_bytes(&wire(0x22, &selection[..size])).is_err());
    }
    popup.possible_values = Some(string(b"Yes;No"));
    let mut both = selection.to_vec();
    both.extend_from_slice(b"Yes;No\0");
    let expected = screen(popup);
    check_fixture(0x22, &both, expected.clone());
    both.extend_from_slice(&[0xA5, 0xFF]);
    assert_eq!(
        DirectCommands::from_bytes(&wire(0x22, &both))
            .unwrap()
            .payload,
        expected
    );
    // Unterminated optional strings are rejected without reading CRC bytes.
    assert!(DirectCommands::from_bytes(&wire(0x22, b"\x01H\0I\0\x05\x01\0Yes")).is_err());
}

#[test]
fn screen_size_limits_and_invalid_combinations() {
    let mut popup = popup();
    popup.header = string(&[b'H'; 51]);
    popup.info = string(b"");
    let packet = DirectCommands {
        dst_addr: 0xEE,
        src_addr: 0xEA,
        payload: screen(popup.clone()),
    };
    let mut buffer = [0; 128];
    assert_eq!(
        write_packet_to_buffer(&mut buffer, PacketAddress::FlightController, &packet),
        Ok(64)
    );
    assert_eq!(DirectCommands::from_bytes(&buffer[3..63]), Ok(packet));
    popup.additional_data = Some(ScreenAdditionalData::Empty);
    assert_eq!(
        ScreenPopup::new(&popup),
        Err(CrsfParsingError::InvalidPayloadLength)
    );
    let mut body = vec![1];
    body.extend_from_slice(&[b'H'; 51]);
    body.extend_from_slice(&[0, 0, 5, 1, 0]);
    assert_eq!(
        DirectCommands::from_bytes(&wire(0x22, &body)),
        Err(CrsfParsingError::InvalidPayloadLength)
    );

    let mut popup = self::popup();
    popup.possible_values = Some(string(b"Yes"));
    assert_eq!(
        ScreenPopup::new(&popup),
        Err(CrsfParsingError::InvalidPayload)
    );
    popup.additional_data = Some(ScreenAdditionalData::Selection(ScreenSelection {
        selection_text: string(b""),
        value: 0,
        min_value: 0,
        max_value: 100,
        default_value: 0,
        unit: string(b""),
    }));
    assert_eq!(
        ScreenPopup::new(&popup),
        Err(CrsfParsingError::InvalidPayload)
    );
    assert_eq!(
        CommandString::new(b"A\0B"),
        Err(CrsfParsingError::InvalidPayload)
    );
    assert_eq!(
        CommandString::new(&[b'A'; 52]),
        Err(CrsfParsingError::InvalidPayloadLength)
    );
    assert_eq!(string(&[0xFF]).as_bytes(), &[0xFF]);
}

#[test]
fn invalid_values_and_reserved_commands() {
    for (id, body) in [
        (3, &b"\x02\x02"[..]),
        (0x0A, &b"\x71\0\x02"[..]),
        (0x22, &b"\x02\0\x02"[..]),
        (0x22, &b"\x01\0\0\0\x02"[..]),
        (9, &b"\x02\xB4\0\0"[..]), // Hue 360
        (9, &b"\x02\0\x65\0"[..]), // Saturation 101
        (9, &b"\x02\0\0\x65"[..]), // Value 101
        (0x0A, &b"\x04"[..]),
        (0x10, &b"\x08"[..]),
        (0x10, &b"\x09"[..]),
        (0x22, &b"\x03"[..]),
        (0x22, &b"\x04"[..]),
        (0x12, &b"\x01"[..]),
        (8, &b"\x01"[..]),
        (8, &b"\x03"[..]),
    ] {
        assert_eq!(
            DirectCommands::from_bytes(&wire(id, body)),
            Err(CrsfParsingError::InvalidPayload)
        );
    }
    for color in [
        LedColor {
            hue: 360,
            saturation: 0,
            value: 0,
        },
        LedColor {
            hue: 0,
            saturation: 101,
            value: 0,
        },
        LedColor {
            hue: 0,
            saturation: 0,
            value: 101,
        },
    ] {
        let packet = DirectCommands {
            dst_addr: 0,
            src_addr: 0,
            payload: CommandPayload::Led(LedCommand::OverrideColor(color)),
        };
        assert_eq!(
            packet.to_bytes(&mut [0; 64]),
            Err(CrsfParsingError::InvalidPayload)
        );
    }
}

#[test]
fn existing_command_types_are_public() {
    // Construct and match the original command families from a downstream crate.
    let payloads = [
        CommandPayload::Fc(FcCommand::ForceDisarm),
        CommandPayload::Osd(OsdCommand::SendButtons(0x80)),
        CommandPayload::Vtx(VtxCommand::SetFrequency(5800)),
        CommandPayload::FlowControl(FlowControlCommand::Unsubscribe { frame_type: 0x14 }),
        CommandPayload::Ack(CommandAck::new(0x10, 1, 1, b"OK").unwrap()),
    ];
    for payload in payloads {
        let packet = DirectCommands {
            dst_addr: 0xEE,
            src_addr: 0xEA,
            payload,
        };
        let mut buffer = [0; 64];
        let len = packet.to_bytes(&mut buffer).unwrap();
        let decoded = DirectCommands::from_bytes(&buffer[..len]).unwrap();
        assert_eq!(decoded, packet);
        if let CommandPayload::Ack(ack) = decoded.payload {
            assert_eq!(ack.information(), b"OK");
        }
    }
}
