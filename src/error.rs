#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CrsfParsingError {
    UnexpectedPacketType(u8),
    PacketNotImlemented(u8),
    InvalidPayloadLength,
    InvalidPayload,
    BufferOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CrsfStreamError {
    InvalidSync,
    InvalidPacketLength,
    InvalidCrc { calculated_crc: u8, packet_crc: u8 },
    UnexpectedPacketType(u8),
    ParsingError(CrsfParsingError),
}
