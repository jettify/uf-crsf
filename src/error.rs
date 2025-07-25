#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrsfParsingError {
    UnexpectedPacketType(u8),
    PacketNotImlemented(u8),
    InvalidPayloadLength,
    InvalidPayload,
    BufferOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrsfStreamError {
    InvalidSync,
    InvalidPacketLength,
    InvalidCrc { calculated_crc: u8, packet_crc: u8 },
    UnexpectedPacketType(u8),
    ParsingError(CrsfParsingError),
}
