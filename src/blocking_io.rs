use crate::error::CrsfStreamError;
use crate::packets::{write_packet_to_buffer, CrsfPacket, Packet, PacketAddress};
use crate::parser::CrsfParser;
use embedded_io::{Error, Read, Write};
use heapless::Deque;

const BLOCKING_IO_BUFFER_SIZE: usize = crate::constants::CRSF_MAX_PACKET_SIZE * 2;

pub struct BlockingCrsfReader<R> {
    parser: CrsfParser,
    reader: R,
    input_buffer: Deque<u8, BLOCKING_IO_BUFFER_SIZE>,
}

impl<R: Read> BlockingCrsfReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            parser: CrsfParser::new(),
            reader,
            input_buffer: Deque::new(),
        }
    }

    pub fn read_packet(&mut self) -> Result<Packet, CrsfStreamError> {
        let mut temp_read_buf = [0; crate::constants::CRSF_MAX_PACKET_SIZE];

        loop {
            while let Some(byte) = self.input_buffer.pop_front() {
                match self.parser.push_byte(byte) {
                    Ok(Some(packet)) => return Ok(packet),
                    Ok(None) => (),
                    Err(e) => return Err(e),
                }
            }
            let bytes_read = self
                .reader
                .read(&mut temp_read_buf)
                .map_err(|e| CrsfStreamError::Io(e.kind()))?;

            if bytes_read == 0 {
                return Err(CrsfStreamError::UnexpectedEof);
            }

            for byte in &temp_read_buf[..bytes_read] {
                self.input_buffer
                    .push_back(*byte)
                    .map_err(|_| CrsfStreamError::InputBufferTooSmall)?;
            }
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
