#[cfg(any(feature = "embedded_io_async", feature = "embedded_io"))]
use embedded_io::ErrorKind;

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
    InvalidPacketLength(u8),
    InvalidSync(u8),
    InvalidCrc {
        calculated_crc: u8,
        packet_crc: u8,
    },
    UnexpectedPacketType(u8),
    ParsingError(CrsfParsingError),
    InputBufferTooSmall,
    #[cfg(any(feature = "embedded_io_async", feature = "embedded_io"))]
    Io(ErrorKind),
    #[cfg(any(feature = "embedded_io_async", feature = "embedded_io"))]
    UnexpectedEof,
}

impl From<CrsfParsingError> for CrsfStreamError {
    fn from(e: CrsfParsingError) -> Self {
        CrsfStreamError::ParsingError(e)
    }
}
