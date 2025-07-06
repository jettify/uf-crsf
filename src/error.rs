#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrsfParsingError {
    UnexpectedPacketType(u8),
    PacketNotImlemented(u8),
    InvalidPayloadLength,
    InvalidPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrsfError {
    InvalidSync,
    InvalidPacketLength,
    InvalidCrc { calculated_crc: u8, packet_crc: u8 },
    UnexpectedPacketType(u8),
    ParsingError(CrsfParsingError),
}