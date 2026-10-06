use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;
use heapless::Vec;

const SUBTYPE_SINGLE: u8 = 0xF0;
const SUBTYPE_STATUS_TEXT: u8 = 0xF1;
const SUBTYPE_MULTI: u8 = 0xF2;

const TELEMETRY_ITEM_SIZE: usize = 6;
const MAX_PACKETS: usize = 9;
const MAX_TEXT_BYTES: usize = 50;

const SINGLE_HEADER_SIZE: usize = 1;
const MULTI_HEADER_SIZE: usize = 2;
const STATUS_TEXT_HEADER_SIZE: usize = 2;
const SINGLE_PAYLOAD_SIZE: usize = SINGLE_HEADER_SIZE + TELEMETRY_ITEM_SIZE;
const MAX_STATUS_TEXT_PAYLOAD_SIZE: usize = STATUS_TEXT_HEADER_SIZE + MAX_TEXT_BYTES;

/// An opaque ArduPilot passthrough telemetry item.
/// Unlike most CRSF fields, `appid` and `data` use little-endian wire encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PassthroughTelemetryPacket {
    pub appid: u16,
    pub data: u32,
}

impl PassthroughTelemetryPacket {
    fn read(data: &[u8]) -> Self {
        Self {
            appid: u16::from_le_bytes([data[0], data[1]]),
            data: u32::from_le_bytes([data[2], data[3], data[4], data[5]]),
        }
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer[..2].copy_from_slice(&self.appid.to_le_bytes());
        buffer[2..TELEMETRY_ITEM_SIZE].copy_from_slice(&self.data.to_le_bytes());
    }
}

/// Status text with an opaque severity and up to 50 raw text bytes.
/// ArduPilot terminates shorter fields with NUL; ExpressLRS may fill all 50 bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PassthroughStatusText {
    pub severity: u8,
    text: Vec<u8, MAX_TEXT_BYTES>,
}

impl PassthroughStatusText {
    /// Creates status text without a terminator. Embedded NUL bytes are invalid.
    pub fn new(severity: u8, text: &[u8]) -> Result<Self, CrsfParsingError> {
        if text.len() > MAX_TEXT_BYTES {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }
        if text.contains(&0) {
            return Err(CrsfParsingError::InvalidPayload);
        }
        let mut bytes = Vec::new();
        bytes
            .extend_from_slice(text)
            .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self {
            severity,
            text: bytes,
        })
    }

    /// Returns the text bytes, excluding the wire terminator and padding.
    pub fn text_bytes(&self) -> &[u8] {
        &self.text
    }

    /// Interprets the text as UTF-8, if valid.
    pub fn text(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.text_bytes())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for PassthroughStatusText {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "PassthroughStatusText {{ severity: {}, text: {:?} }}",
            self.severity,
            self.text_bytes()
        )
    }
}

/// ArduPilot broadcast passthrough payload (frame type 0x80).
/// No destination or origin bytes appear inside the payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArduPilotPassthrough {
    /// Subtype 0xF0: one telemetry item.
    Single(PassthroughTelemetryPacket),
    /// Subtype 0xF2: zero through nine telemetry items.
    Multi(Vec<PassthroughTelemetryPacket, MAX_PACKETS>),
    /// Subtype 0xF1: severity and text.
    StatusText(PassthroughStatusText),
}

impl ArduPilotPassthrough {
    /// Creates a batch of at most nine telemetry items.
    pub fn multi(packets: &[PassthroughTelemetryPacket]) -> Result<Self, CrsfParsingError> {
        let mut items = Vec::new();
        items
            .extend_from_slice(packets)
            .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
        Ok(Self::Multi(items))
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ArduPilotPassthrough {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Self::Single(packet) => defmt::write!(fmt, "Single({})", packet),
            Self::Multi(packets) => defmt::write!(fmt, "Multi({:?})", packets.as_slice()),
            Self::StatusText(text) => defmt::write!(fmt, "StatusText({})", text),
        }
    }
}

impl CrsfPacket for ArduPilotPassthrough {
    const PACKET_TYPE: PacketType = PacketType::ArdupilotResponse;
    const MIN_PAYLOAD_SIZE: usize = MULTI_HEADER_SIZE;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        let subtype = *data.first().ok_or(CrsfParsingError::InvalidPayloadLength)?;
        match subtype {
            SUBTYPE_SINGLE => {
                if data.len() < SINGLE_PAYLOAD_SIZE {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                Ok(Self::Single(PassthroughTelemetryPacket::read(
                    &data[SINGLE_HEADER_SIZE..SINGLE_PAYLOAD_SIZE],
                )))
            }
            SUBTYPE_MULTI => {
                if data.len() < MULTI_HEADER_SIZE {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                let count = data[1] as usize;
                if count > MAX_PACKETS
                    || data.len() < MULTI_HEADER_SIZE + count * TELEMETRY_ITEM_SIZE
                {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                let mut packets = Vec::new();
                for item in data[MULTI_HEADER_SIZE..MULTI_HEADER_SIZE + count * TELEMETRY_ITEM_SIZE]
                    .chunks_exact(TELEMETRY_ITEM_SIZE)
                {
                    packets
                        .push(PassthroughTelemetryPacket::read(item))
                        .map_err(|_| CrsfParsingError::InvalidPayloadLength)?;
                }
                Ok(Self::Multi(packets))
            }
            SUBTYPE_STATUS_TEXT => {
                if data.len() < STATUS_TEXT_HEADER_SIZE + 1 {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                let field =
                    &data[STATUS_TEXT_HEADER_SIZE..data.len().min(MAX_STATUS_TEXT_PAYLOAD_SIZE)];
                let text_len = match field.iter().position(|&byte| byte == 0) {
                    Some(len) => len,
                    None if field.len() == MAX_TEXT_BYTES => MAX_TEXT_BYTES,
                    None => return Err(CrsfParsingError::InvalidPayload),
                };
                Ok(Self::StatusText(PassthroughStatusText::new(
                    data[1],
                    &field[..text_len],
                )?))
            }
            _ => Err(CrsfParsingError::InvalidPayload),
        }
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        let payload_len = match self {
            Self::Single(_) => SINGLE_PAYLOAD_SIZE,
            Self::Multi(packets) => MULTI_HEADER_SIZE + packets.len() * TELEMETRY_ITEM_SIZE,
            Self::StatusText(text) => {
                STATUS_TEXT_HEADER_SIZE
                    + text.text.len()
                    + usize::from(text.text.len() < MAX_TEXT_BYTES)
            }
        };
        if buffer.len() < payload_len {
            return Err(CrsfParsingError::BufferOverflow);
        }
        match self {
            Self::Single(packet) => {
                buffer[0] = SUBTYPE_SINGLE;
                packet.write(&mut buffer[SINGLE_HEADER_SIZE..SINGLE_PAYLOAD_SIZE]);
            }
            Self::Multi(packets) => {
                buffer[0] = SUBTYPE_MULTI;
                buffer[1] = packets.len() as u8;
                for (packet, chunk) in packets.iter().zip(
                    buffer[MULTI_HEADER_SIZE..payload_len].chunks_exact_mut(TELEMETRY_ITEM_SIZE),
                ) {
                    packet.write(chunk);
                }
            }
            Self::StatusText(text) => {
                buffer[0] = SUBTYPE_STATUS_TEXT;
                buffer[1] = text.severity;
                buffer[STATUS_TEXT_HEADER_SIZE..STATUS_TEXT_HEADER_SIZE + text.text.len()]
                    .copy_from_slice(text.text_bytes());
                if text.text.len() < MAX_TEXT_BYTES {
                    buffer[payload_len - 1] = 0;
                }
            }
        }
        Ok(payload_len)
    }
}

/// Legacy frame type 0x7F, carrying the same payload as frame type 0x80.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ArduPilotLegacy(pub ArduPilotPassthrough);

impl CrsfPacket for ArduPilotLegacy {
    const PACKET_TYPE: PacketType = PacketType::ArdupilotLegacy;
    const MIN_PAYLOAD_SIZE: usize = ArduPilotPassthrough::MIN_PAYLOAD_SIZE;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        ArduPilotPassthrough::from_bytes(data).map(Self)
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.0.to_bytes(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEM: PassthroughTelemetryPacket = PassthroughTelemetryPacket {
        appid: 0x1234,
        data: 0x12345678,
    };

    #[test]
    fn payload_fixtures_and_extensions() {
        let cases = [
            (
                ArduPilotPassthrough::Single(ITEM),
                &[0xf0, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12][..],
            ),
            (
                ArduPilotPassthrough::multi(&[ITEM]).unwrap(),
                &[0xf2, 1, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12][..],
            ),
            (
                ArduPilotPassthrough::StatusText(PassthroughStatusText::new(4, b"ABC").unwrap()),
                &[0xf1, 4, b'A', b'B', b'C', 0][..],
            ),
        ];
        for (packet, expected) in cases {
            let mut buffer = [0xaa; 60];
            let len = packet.to_bytes(&mut buffer).unwrap();
            assert_eq!(&buffer[..len], expected);
            assert_eq!(buffer[len], 0xaa);
            assert_eq!(
                ArduPilotPassthrough::from_bytes(expected),
                Ok(packet.clone())
            );
            assert_eq!(
                ArduPilotLegacy::from_bytes(expected),
                Ok(ArduPilotLegacy(packet.clone()))
            );
            assert_eq!(
                ArduPilotPassthrough::from_bytes(&buffer[..len + 3]),
                Ok(packet.clone())
            );
            for size in 0..len {
                let mut short = [0xaa; 60];
                assert_eq!(
                    packet.to_bytes(&mut short[..size]),
                    Err(CrsfParsingError::BufferOverflow)
                );
                assert_eq!(short, [0xaa; 60]);
                // All known layouts require every byte, including the text terminator.
                assert!(ArduPilotPassthrough::from_bytes(&expected[..size]).is_err());
            }
        }
    }

    #[test]
    fn batch_boundaries() {
        for count in [0, 9] {
            let packet = ArduPilotPassthrough::multi(&[ITEM; 9][..count]).unwrap();
            let mut bytes = [0; 56];
            let len = packet.to_bytes(&mut bytes).unwrap();
            assert_eq!(len, 2 + count * 6);
            assert_eq!(bytes[1], count as u8);
            assert_eq!(ArduPilotPassthrough::from_bytes(&bytes[..len]), Ok(packet));
        }
        assert_eq!(
            ArduPilotPassthrough::multi(&[ITEM; 10]),
            Err(CrsfParsingError::InvalidPayloadLength)
        );
        for payload in [&[0xf2, 10][..], &[0xf2, 255], &[0xf2, 1], &[0xf2]] {
            assert_eq!(
                ArduPilotPassthrough::from_bytes(payload),
                Err(CrsfParsingError::InvalidPayloadLength)
            );
        }
    }

    #[test]
    fn status_text_boundaries_and_raw_bytes() {
        for text in [&[][..], &[b'x'; 49], &[b'x'; 50], &[0xff, 0xfe]] {
            let status = PassthroughStatusText::new(255, text).unwrap();
            assert_eq!(status.text_bytes(), text);
            assert_eq!(status.text().is_ok(), core::str::from_utf8(text).is_ok());
            let packet = ArduPilotPassthrough::StatusText(status);
            let mut buffer = [0xaa; 60];
            let len = packet.to_bytes(&mut buffer).unwrap();
            assert_eq!(len, 2 + text.len() + usize::from(text.len() < 50));
            assert_eq!(
                ArduPilotPassthrough::from_bytes(&buffer[..len]),
                Ok(packet.clone())
            );
            assert_eq!(
                ArduPilotPassthrough::from_bytes(&buffer[..len + 3]),
                Ok(packet)
            );
        }
        let mut padded = [0; 52];
        padded[..5].copy_from_slice(&[0xf1, 4, b'A', b'B', 0]);
        assert_eq!(
            ArduPilotPassthrough::from_bytes(&padded),
            Ok(ArduPilotPassthrough::StatusText(
                PassthroughStatusText::new(4, b"AB").unwrap()
            ))
        );
        assert_eq!(
            PassthroughStatusText::new(4, b"ABC").unwrap().text(),
            Ok("ABC")
        );
        assert_eq!(
            PassthroughStatusText::new(0, &[b'x'; 51]),
            Err(CrsfParsingError::InvalidPayloadLength)
        );
        assert_eq!(
            PassthroughStatusText::new(0, b"a\0b"),
            Err(CrsfParsingError::InvalidPayload)
        );
        assert_eq!(
            ArduPilotPassthrough::from_bytes(&[0xf1, 0, b'x']),
            Err(CrsfParsingError::InvalidPayload)
        );
        assert_eq!(
            ArduPilotPassthrough::from_bytes(&[0xf1, 0]),
            Err(CrsfParsingError::InvalidPayloadLength)
        );
        let mut terminator_outside_field = [b'x'; 53];
        terminator_outside_field[..2].copy_from_slice(&[0xf1, 0]);
        terminator_outside_field[52] = 0;
        assert_eq!(
            ArduPilotPassthrough::from_bytes(&terminator_outside_field),
            Ok(ArduPilotPassthrough::StatusText(
                PassthroughStatusText::new(0, &[b'x'; 50]).unwrap()
            ))
        );
    }

    #[test]
    fn unknown_subtypes_and_empty_payload() {
        assert_eq!(
            ArduPilotPassthrough::from_bytes(&[]),
            Err(CrsfParsingError::InvalidPayloadLength)
        );
        for subtype in 0..=255 {
            if !matches!(subtype, 0xf0..=0xf2) {
                assert_eq!(
                    ArduPilotPassthrough::from_bytes(&[subtype, 0]),
                    Err(CrsfParsingError::InvalidPayload)
                );
            }
        }
    }
}
