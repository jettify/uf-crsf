use crate::error::CrsfStreamError;
use crate::packets::{write_packet_to_buffer, CrsfPacket, Packet, PacketAddress};
use crate::parser::{CrsfParser, ParseResult};
use embedded_io::{Error, Read, Write};

impl CrsfParser {
    /// Synchronously reads a complete CRSF packet from an `embedded_io::Read` stream.
    ///
    /// This function reads bytes in chunks from the provided `reader` and pushes them
    /// into the parser one byte at time.
    pub fn read_packet_blocking<R: Read>(
        &mut self,
        reader: &mut R,
    ) -> Result<Packet, CrsfStreamError> {
        let mut buf = [0; 64]; // 64 is max packet size for CRSF
        loop {
            let n = reader
                .read(&mut buf)
                .map_err(|e| CrsfStreamError::Io(e.kind()))?;
            if n == 0 {
                // This indicates a stream has closed.
                return Err(CrsfStreamError::UnexpectedEof);
            }

            for b in &buf[0..n] {
                match self.push_byte(*b) {
                    ParseResult::Complete(packet) => return Ok(packet),
                    ParseResult::Incomplete => continue,
                    ParseResult::Error(e) => return Err(e),
                }
            }
        }
    }
}

/// Synchronously writes a CRSF packet to an `embedded_io::Write` stream.
///
/// This function serializes the given `packet` into a temporary buffer and then
/// writes the entire buffer to the `writer` synchronously.
pub fn write_packet_blocking<W: Write, P: CrsfPacket>(
    writer: &mut W,
    dest: PacketAddress,
    packet: &P,
) -> Result<(), CrsfStreamError> {
    let mut buffer = [0u8; crate::constants::CRSF_MAX_PACKET_SIZE];
    let len = write_packet_to_buffer(&mut buffer, dest, packet)?;
    writer
        .write_all(&buffer[..len])
        .map_err(|e| CrsfStreamError::Io(e.kind()))?;
    Ok(())
}
