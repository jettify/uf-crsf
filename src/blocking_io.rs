use crate::error::CrsfStreamError;
use crate::packets::write_packet_to_buffer;
use crate::packets::CrsfPacket;
use crate::packets::Packet;
use crate::packets::PacketAddress;
use crate::parser::CrsfParser;
use embedded_io::Error;
use embedded_io::Read;
use embedded_io::Write;

pub struct BlockingCrsfReader<R> {
    parser: CrsfParser,
    reader: R,
    input_buffer: [u8; crate::constants::CRSF_MAX_PACKET_SIZE],
    start: usize,
    end: usize,
}

impl<R: Read> BlockingCrsfReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            parser: CrsfParser::new(),
            reader,
            input_buffer: [0; crate::constants::CRSF_MAX_PACKET_SIZE],
            start: 0,
            end: 0,
        }
    }

    pub fn read_packet(&mut self) -> Result<Packet, CrsfStreamError> {
        loop {
            let mut input = &self.input_buffer[self.start..self.end];
            let result = self.parser.next_packet(&mut input);
            self.start = self.end - input.len();
            if let Some(result) = result {
                return result;
            }

            self.start = 0;
            self.end = 0;
            let bytes_read = self
                .reader
                .read(&mut self.input_buffer)
                .map_err(|e| CrsfStreamError::Io(e.kind()))?;
            if bytes_read == 0 {
                return Err(CrsfStreamError::UnexpectedEof);
            }
            self.end = bytes_read;
        }
    }
}

/// Synchronously writes a CRSF packet to an `embedded_io::Write` stream.
///
/// This function serializes the given `packet` into a temporary buffer and then
/// writes the entire buffer to the `writer` synchronously.
pub fn write_packet<W: Write, P: CrsfPacket>(
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
