#![no_std]
#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]

pub mod constants;
pub mod error;
pub mod packets;
pub mod parser;

pub use error::{CrsfParsingError, CrsfStreamError};
pub use packets::{write_packet_to_buffer, Packet, PacketAddress, PacketType};
pub use parser::{CrsfParser, RawCrsfPacket};
