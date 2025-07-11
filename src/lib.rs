#![no_std]

pub mod constants;
pub mod error;
pub mod packets;
pub mod parser;

pub use error::{CrsfParsingError, CrsfStreamError};
pub use packets::{write_packet_to_buffer, Packet, PacketAddress, PacketType};
pub use parser::{CrsfParser, RawCrsfPacket};
