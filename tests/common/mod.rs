use core::cell::Cell;
use std::rc::Rc;

pub const VALID_FRAME: [u8; 14] = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];

pub struct ChunkReader<'a> {
    pub input: &'a [u8],
    pub chunk_size: usize,
    pub reads: Rc<Cell<usize>>,
    pub fail_on: Option<usize>,
}

impl embedded_io::ErrorType for ChunkReader<'_> {
    type Error = embedded_io::ErrorKind;
}

impl ChunkReader<'_> {
    fn read_chunk(&mut self, buffer: &mut [u8]) -> Result<usize, embedded_io::ErrorKind> {
        let reads = self.reads.get() + 1;
        self.reads.set(reads);
        if self.fail_on == Some(reads) {
            return Err(embedded_io::ErrorKind::Other);
        }
        let count = self.input.len().min(self.chunk_size).min(buffer.len());
        buffer[..count].copy_from_slice(&self.input[..count]);
        self.input = &self.input[count..];
        Ok(count)
    }
}

#[cfg(feature = "embedded_io")]
impl embedded_io::Read for ChunkReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_chunk(buffer)
    }
}

#[cfg(feature = "embedded_io_async")]
impl embedded_io_async::Read for ChunkReader<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_chunk(buffer)
    }
}

// A rejected 32-byte candidate containing two complete frames. With 16-byte
// reads, both frames are retained when the outer CRC fails on the second read.
pub fn recovery_stream() -> Vec<u8> {
    let mut bytes = vec![0xC0, 30, 0xFF];
    bytes.extend_from_slice(&VALID_FRAME);
    bytes.extend_from_slice(&VALID_FRAME);
    let crc = crc::Crc::<u8>::new(&crc::CRC_8_DVB_S2).checksum(&bytes[2..]);
    bytes.push(if crc == 0xFF { 0xFE } else { 0xFF });
    bytes
}
