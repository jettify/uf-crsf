use uf_crsf::packets::*;
use uf_crsf::{CrsfParser, CrsfParsingError, CrsfStreamError};

fn frame(packet_type: PacketType, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0xc8, (payload.len() + 2) as u8, packet_type as u8];
    bytes.extend_from_slice(payload);
    let crc = crc::Crc::<u8>::new(&crc::CRC_8_DVB_S2).checksum(&bytes[2..]);
    bytes.push(crc);
    bytes
}

fn check_extensions<T: CrsfPacket + core::fmt::Debug + PartialEq>(
    payload: &[u8],
    wrap: fn(T) -> Packet,
) {
    let expected = T::from_bytes(payload).unwrap();
    let expected_packet = wrap(T::from_bytes(payload).unwrap());
    // Cover every legal extension length, including a maximum-size frame.
    let mut extended = payload.to_vec();
    for len in payload.len()..=60 {
        assert_eq!(T::from_bytes(&extended).unwrap(), expected);
        assert_eq!(
            CrsfParser::new()
                .iter_packets(&frame(T::PACKET_TYPE, &extended))
                .collect::<Vec<_>>(),
            vec![Ok(expected_packet.clone())],
            "{:?}, payload length {len}",
            T::PACKET_TYPE,
        );
        extended.push(0xa5);
    }

    // Extensions are discarded when encoding the decoded packet.
    let decoded = T::from_bytes(&extended[..60]).unwrap();
    let mut encoded = [0; 60];
    let size = decoded.to_bytes(&mut encoded).unwrap();
    assert_eq!(&encoded[..size], payload);

    for len in 0..T::MIN_PAYLOAD_SIZE {
        assert_eq!(
            T::from_bytes(&payload[..len]),
            Err(CrsfParsingError::InvalidPayloadLength),
        );
        assert_eq!(
            CrsfParser::new()
                .iter_packets(&frame(T::PACKET_TYPE, &payload[..len]))
                .collect::<Vec<_>>(),
            vec![Err(CrsfStreamError::ParsingError(
                CrsfParsingError::InvalidPayloadLength,
            ))],
        );
    }
}

#[test]
fn fixed_size_codecs_accept_extensions_and_reject_truncation() {
    macro_rules! check {
        ($($codec:ty => $variant:ident),+ $(,)?) => {
            $(
                let payload: Vec<_> = (1..=<$codec>::MIN_PAYLOAD_SIZE as u8).collect();
                check_extensions::<$codec>(&payload, Packet::$variant);
            )+
        };
    }
    check!(
        AccelGyro => AccelGyro,
        AirSpeed => AirSpeed,
        Attitude => Attitude,
        BaroAltitude => BaroAltitude,
        Barometer => Barometer,
        EspNow => EspNow,
        Gps => Gps,
        GpsExtended => GpsExtended,
        GpsTime => GpsTime,
        Heartbeat => Heartbeat,
        LinkStatistics => LinkStatistics,
        LinkStatisticsRepeater => LinkStatisticsRepeater,
        Magnetometer => Magnetometer,
        MavLinkFc => MavLinkFc,
        MavLinkSensor => MavLinkSensor,
        RcChannelsPacked => RCChannels,
        VariometerSensor => Vario,
        VtxTelemetry => VtxTelemetry,
        DevicePing => DevicePing,
    );
    check_extensions::<Game>(&[0xea, 0xec, 1, 0, 42], Packet::Game);
    check_extensions::<Remote>(&[0xea, 0xec, 0x10, 0, 0, 0, 1, 0, 0, 0, 2], Packet::Remote);
}

#[test]
fn antenna_extensions_are_decoded_before_unknown_trailing_fields() {
    check_extensions::<LinkStatisticsRx>(&[100, 75, 90, 246, 20, 110, 1], Packet::LinkStatisticsRx);
    check_extensions::<LinkStatisticsTx>(
        &[100, 75, 90, 246, 20, 50, 110, 1],
        Packet::LinkStatisticsTx,
    );
    // A partial known antenna pair remains invalid even with a valid frame CRC.
    for (packet_type, payload) in [
        (
            PacketType::LinkStatisticsRx,
            &[100, 75, 90, 246, 20, 110][..],
        ),
        (
            PacketType::LinkStatisticsTx,
            &[100, 75, 90, 246, 20, 50, 110][..],
        ),
    ] {
        assert_eq!(
            CrsfParser::new()
                .iter_packets(&frame(packet_type, payload))
                .collect::<Vec<_>>(),
            vec![Err(CrsfStreamError::ParsingError(
                CrsfParsingError::InvalidPayloadLength,
            ))],
        );
    }
}
