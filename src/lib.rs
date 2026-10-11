#![no_std]
#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]

pub mod constants;
pub mod error;
pub mod packets;
pub mod parser;

#[cfg(feature = "embedded_io_async")]
pub mod async_io;

#[cfg(feature = "embedded_io")]
pub mod blocking_io;

pub use error::CrsfParsingError;
pub use error::CrsfStreamError;
pub use packets::write_packet_to_buffer;
pub use packets::Packet;
pub use packets::PacketAddress;
pub use packets::PacketType;
pub use parser::CrsfParser;
pub use parser::RawCrsfPacket;
