use crate::constants;
use crate::error::CrsfStreamError;
use crate::packets::Packet;
use crate::packets::PacketAddress;
use crc::Crc;
use num_enum::TryFromPrimitive;

#[derive(Debug, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    #[default]
    AwaitingSync,
    AwaitingLength,
    Reading {
        frame_len: usize,
    },
    AwaitingCrc,
}

#[derive(Debug)]
pub struct CrsfParser {
    buffer: [u8; constants::CRSF_MAX_PACKET_SIZE],
    state: State,
    len: usize,
}

const CRC8_DVB_S2: Crc<u8> = Crc::<u8>::new(&crc::CRC_8_DVB_S2);

impl CrsfParser {
    pub fn new() -> Self {
        Self {
            buffer: [0; constants::CRSF_MAX_PACKET_SIZE],
            state: State::AwaitingSync,
            len: 0,
        }
    }

    pub fn push_byte_raw(
        &mut self,
        byte: u8,
    ) -> Result<Option<RawCrsfPacket<'_>>, CrsfStreamError> {
        Ok(self.push_byte_frame(byte)?.map(|frame_len| RawCrsfPacket {
            bytes: &self.buffer[..frame_len],
        }))
    }

    pub fn push_byte(&mut self, byte: u8) -> Result<Option<Packet>, CrsfStreamError> {
        match self.push_byte_raw(byte) {
            Ok(Some(raw_packet)) => Packet::parse(&raw_packet)
                .map(Some)
                .map_err(CrsfStreamError::ParsingError),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Iterates over owned, decoded packets from an input slice.
    ///
    /// Uses [`Self::next_raw_packet`] to avoid buffering contiguous frames,
    /// then decodes each frame into an owned [`Packet`]. Exhausting the iterator
    /// preserves partial frames for later chunks. To stop early without losing
    /// unread bytes, inspect [`PacketIterator::remaining`] or use
    /// [`Self::next_packet`] directly with a mutable slice cursor.
    pub fn iter_packets<'a, 'b>(&'a mut self, input: &'b [u8]) -> PacketIterator<'a, 'b> {
        PacketIterator {
            parser: self,
            remaining: input,
        }
    }

    /// Advances `input` to the next decoded [`Packet`] or stream/parsing error.
    ///
    /// Contiguous frames are decoded directly from `*input` without copying into
    /// the parser's internal buffer; frames split across chunks use the internal
    /// buffer. Returns `None` once `input` is exhausted (buffering any trailing
    /// partial frame).
    pub fn next_packet(&mut self, input: &mut &[u8]) -> Option<Result<Packet, CrsfStreamError>> {
        self.next_raw_packet(input).map(|result| {
            result.and_then(|raw| Packet::parse(&raw).map_err(CrsfStreamError::ParsingError))
        })
    }

    /// Advances `input` to the next validated [`RawCrsfPacket`] or framing error.
    ///
    /// Contiguous frames borrow directly from `*input`; frames split across chunks
    /// borrow from the parser's internal spillover buffer. Returns `None` once
    /// `input` is exhausted (buffering any trailing partial frame for subsequent
    /// calls).
    ///
    /// Release each borrowed frame before calling `next_raw_packet` again:
    ///
    /// ```
    /// use uf_crsf::CrsfParser;
    ///
    /// let mut parser = CrsfParser::new();
    /// let buf = [0xC8, 2, 0, 0, 0xC8, 2, 0, 0];
    /// let mut input = buf.as_slice();
    ///
    /// let first = parser.next_raw_packet(&mut input).unwrap().unwrap();
    /// assert_eq!(first.len(), 4);
    /// assert_eq!(input, &[0xC8, 2, 0, 0]);
    ///
    /// let second = parser.next_raw_packet(&mut input).unwrap().unwrap();
    /// assert_eq!(second.len(), 4);
    /// assert!(parser.next_raw_packet(&mut input).is_none());
    /// assert!(input.is_empty());
    /// ```
    ///
    /// Holding a frame across a subsequent read fails to compile because split
    /// frames may borrow the parser's internal buffer:
    ///
    /// ```compile_fail,E0499
    /// use uf_crsf::CrsfParser;
    ///
    /// let mut parser = CrsfParser::new();
    /// let buf = [0xC8, 2, 0, 0, 0xC8, 2, 0, 0];
    /// let mut input = buf.as_slice();
    /// let first = parser.next_raw_packet(&mut input).unwrap().unwrap();
    /// let second = parser.next_raw_packet(&mut input);
    /// assert_eq!(first.len(), 4);
    /// ```
    pub fn next_raw_packet<'p, 'i: 'p>(
        &'p mut self,
        input: &mut &'i [u8],
    ) -> Option<Result<RawCrsfPacket<'p>, CrsfStreamError>> {
        while !input.is_empty() {
            if self.state == State::AwaitingSync {
                let chunk = *input;
                let sync_byte = chunk[0];
                if !Self::is_valid_sync(sync_byte) {
                    *input = &chunk[1..];
                    return Some(Err(CrsfStreamError::InvalidSync(sync_byte)));
                }
                if chunk.len() == 1 {
                    self.try_start_frame(sync_byte);
                    *input = &[];
                    return None;
                }

                let length_byte = chunk[1];
                let frame_len = usize::from(length_byte) + 2;
                if !Self::is_valid_frame_len(frame_len) {
                    // Leave a valid sync candidate at the front of `input` so a
                    // resynchronized frame in this chunk stays on the zero-copy path.
                    *input = if Self::is_valid_sync(length_byte) {
                        &chunk[1..]
                    } else {
                        &chunk[2..]
                    };
                    return Some(Err(CrsfStreamError::InvalidPacketLength(length_byte)));
                }

                if chunk.len() < frame_len {
                    self.buffer_partial_frame(chunk, frame_len);
                    *input = &[];
                    return None;
                }

                let calculated_crc = CRC8_DVB_S2.checksum(&chunk[2..frame_len - 1]);
                let packet_crc = chunk[frame_len - 1];
                if calculated_crc != packet_crc {
                    *input = if Self::is_valid_sync(packet_crc) {
                        &chunk[frame_len - 1..]
                    } else {
                        &chunk[frame_len..]
                    };
                    return Some(Err(CrsfStreamError::InvalidCrc {
                        calculated_crc,
                        packet_crc,
                    }));
                }

                *input = &chunk[frame_len..];
                return Some(Ok(RawCrsfPacket {
                    bytes: &chunk[..frame_len],
                }));
            }

            // Copy only the body bytes needed by this split candidate, leaving
            // its CRC and any following frames for the shared validation path.
            if let State::Reading { frame_len } = self.state {
                let body_remaining = (frame_len - 1) - self.len;
                let copy_len = body_remaining.min(input.len());
                self.buffer[self.len..self.len + copy_len].copy_from_slice(&input[..copy_len]);
                self.len += copy_len;
                *input = &input[copy_len..];
                if self.len == frame_len - 1 {
                    self.state = State::AwaitingCrc;
                }
                continue;
            }

            // Headers and CRCs of split candidates share the byte state machine.
            let chunk = *input;
            let byte = chunk[0];
            *input = &chunk[1..];
            match self.push_byte_frame(byte) {
                Ok(Some(frame_len)) => {
                    return Some(Ok(RawCrsfPacket {
                        bytes: &self.buffer[..frame_len],
                    }));
                }
                Ok(None) => (),
                Err(error) => {
                    // If `byte` is a valid sync candidate, keep it in `input`
                    // so a contiguous frame starting at `byte` remains zero-copy.
                    if self.state == State::AwaitingLength {
                        self.reset();
                        *input = chunk;
                    }
                    return Some(Err(error));
                }
            }
        }
        None
    }

    pub fn reset(&mut self) {
        self.len = 0;
        self.state = State::AwaitingSync;
    }

    // Return the completed frame's length separately from its borrow so both
    // byte-by-byte and slice-cursor APIs can advance the same state machine.
    fn push_byte_frame(&mut self, byte: u8) -> Result<Option<usize>, CrsfStreamError> {
        match self.state {
            State::AwaitingSync => {
                if self.try_start_frame(byte) {
                    Ok(None)
                } else {
                    Err(CrsfStreamError::InvalidSync(byte))
                }
            }
            State::AwaitingLength => {
                let frame_len = usize::from(byte) + 2;
                if !Self::is_valid_frame_len(frame_len) {
                    self.reset();
                    // A false sync can make this "length" byte invalid. Re-evaluate
                    // the same byte as a new sync candidate so it is not lost.
                    self.try_start_frame(byte);
                    return Err(CrsfStreamError::InvalidPacketLength(byte));
                }
                self.buffer[1] = byte;
                self.len = 2;
                self.state = State::Reading { frame_len };
                Ok(None)
            }
            State::Reading { frame_len } => {
                self.buffer[self.len] = byte;
                self.len += 1;
                if self.len == frame_len - 1 {
                    self.state = State::AwaitingCrc;
                }
                Ok(None)
            }
            State::AwaitingCrc => {
                let calculated_crc = CRC8_DVB_S2.checksum(&self.buffer[2..self.len]);
                let packet_crc = byte;
                if calculated_crc != packet_crc {
                    self.reset();
                    // Preserve this byte as potential sync for the next frame.
                    self.try_start_frame(byte);
                    return Err(CrsfStreamError::InvalidCrc {
                        calculated_crc,
                        packet_crc,
                    });
                }
                self.buffer[self.len] = byte;
                let frame_len = self.len + 1;
                self.reset();
                Ok(Some(frame_len))
            }
        }
    }

    fn is_valid_sync(byte: u8) -> bool {
        PacketAddress::try_from_primitive(byte).is_ok()
    }

    fn is_valid_frame_len(frame_len: usize) -> bool {
        (constants::CRSF_MIN_PACKET_SIZE..=constants::CRSF_MAX_PACKET_SIZE).contains(&frame_len)
    }

    fn try_start_frame(&mut self, byte: u8) -> bool {
        if Self::is_valid_sync(byte) {
            self.buffer[0] = byte;
            self.len = 1;
            self.state = State::AwaitingLength;
            true
        } else {
            false
        }
    }

    fn buffer_partial_frame(&mut self, chunk: &[u8], frame_len: usize) {
        self.buffer[..chunk.len()].copy_from_slice(chunk);
        self.len = chunk.len();
        self.state = if self.len == frame_len - 1 {
            State::AwaitingCrc
        } else {
            State::Reading { frame_len }
        };
    }
}

impl Default for CrsfParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a borrowed, unparsed CRSF packet.
///
/// Parsers validate the framing before returning this view. Direct construction
/// through [`Self::new`] checks only minimum length. Accessors expose the sync
/// byte, type, payload, and CRC without decoding the payload.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawCrsfPacket<'a> {
    bytes: &'a [u8],
}

impl<'a> RawCrsfPacket<'a> {
    /// Creates a new `RawCrsfPacket` from a byte slice.
    ///
    /// Returns `None` if the slice is shorter than the minimum possible
    /// CRSF packet length (4 bytes).
    /// Does not validate the address, declared length, maximum size, or CRC.
    pub fn new(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() >= 4 {
            Some(Self { bytes })
        } else {
            None
        }
    }

    /// Returns the complete raw packet frame slice, including framing and CRC.
    pub fn as_slice(&self) -> &'a [u8] {
        self.bytes
    }

    /// Returns the destination address byte of the packet.
    pub fn dst_addr(&self) -> u8 {
        self.bytes[0]
    }

    /// Returns the raw packet type byte.
    pub fn raw_packet_type(&self) -> u8 {
        self.bytes[2]
    }

    /// Returns a slice representing the packet's payload.
    ///
    /// The payload does not include the CRSF framing (destination, size, type, CRC).
    pub fn payload(&self) -> &'a [u8] {
        &self.bytes[3..self.bytes.len() - 1]
    }

    /// Returns the CRC check byte of the packet.
    #[expect(clippy::missing_panics_doc, reason = "infallible")]
    pub fn crc(&self) -> u8 {
        *self.bytes.last().expect("infallible due to length check")
    }

    /// Returns the total length of the packet in bytes, including the framing.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns `true` if the packet has a length of zero.
    ///
    /// Note: A valid CRSF packet should not be empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl AsRef<[u8]> for RawCrsfPacket<'_> {
    fn as_ref(&self) -> &[u8] {
        self.bytes
    }
}

pub struct PacketIterator<'a, 'b> {
    parser: &'a mut CrsfParser,
    remaining: &'b [u8],
}

impl<'b> PacketIterator<'_, 'b> {
    /// Returns the unread input suffix with the original input lifetime.
    pub fn remaining(&self) -> &'b [u8] {
        self.remaining
    }
}

impl Iterator for PacketIterator<'_, '_> {
    type Item = Result<Packet, CrsfStreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next_packet(&mut self.remaining)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::packets::write_packet_to_buffer;
    use crate::packets::CrsfPacket;
    use crate::packets::LinkStatistics;
    use crate::packets::PacketAddress;
    use crate::packets::PacketType;
    use crate::packets::RcChannelsPacked;

    #[test]
    fn typed_contiguous_frames_do_not_use_spillover_storage() {
        let input = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
        let mut parser = CrsfParser::new();
        parser.buffer.fill(0xA5);
        let packet = parser.iter_packets(&input).next().unwrap().unwrap();
        assert!(matches!(packet, Packet::LinkStatistics(_)));
        assert_eq!(parser.buffer, [0xA5; constants::CRSF_MAX_PACKET_SIZE]);
        assert_eq!(parser.state, State::AwaitingSync);
    }

    #[test]
    fn split_frames_borrow_parser_storage() {
        let input = [0xC8, 3, 0, 0, 0];
        for split in 1..input.len() {
            let mut parser = CrsfParser::new();
            let storage = parser.buffer[3..].as_ptr();
            let mut first_half = &input[..split];
            assert!(parser.next_raw_packet(&mut first_half).is_none());
            assert!(first_half.is_empty());

            let mut second_half = &input[split..];
            let frame = parser.next_raw_packet(&mut second_half).unwrap().unwrap();
            assert_eq!(frame.payload(), &[0]);
            assert_eq!(frame.payload().as_ptr(), storage);
            assert!(parser.next_raw_packet(&mut second_half).is_none());
            assert!(second_half.is_empty());
            assert_eq!(parser.state, State::AwaitingSync);
        }
    }

    #[test]
    fn test_construction() {
        let raw_bytes: [u8; 14] = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
        let mut parser = CrsfParser::new();

        for b in &raw_bytes[0..raw_bytes.len() - 1] {
            let result = parser.push_byte_raw(*b);
            assert!(matches!(result, Ok(None)));
        }

        let raw_packet_result = parser.push_byte_raw(raw_bytes[13]);
        let raw_packet = raw_packet_result
            .expect("Failed to get raw packet result")
            .expect("Expected a complete raw packet, but got None");

        assert_eq!(raw_packet.len(), raw_bytes.len());

        assert_eq!(raw_packet.payload().len(), LinkStatistics::MIN_PAYLOAD_SIZE);
        assert_eq!(
            raw_packet.raw_packet_type(),
            PacketType::LinkStatistics as u8
        );

        let data = &raw_packet.payload();
        let ls = LinkStatistics::from_bytes(data).unwrap();
        let p = Packet::parse(&raw_packet).unwrap();
        assert_eq!(Packet::LinkStatistics(ls.clone()), p);

        assert_eq!(ls.uplink_rssi_1, 16);
    }

    #[test]
    fn test_parsing() {
        let raw_bytes: [u8; 40] = [
            0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252, 0xC8, 24, 0x16, 0xE0, 0x03,
            0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01, 0x0B, 0xF8, 0xC0,
            0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 103,
        ];
        let mut parser = CrsfParser::new();
        let results: std::vec::Vec<Result<Packet, CrsfStreamError>> =
            parser.iter_packets(&raw_bytes).collect();

        let expected = [
            992, 992, 352, 992, 352, 352, 352, 352, 352, 352, 992, 992, 0, 0, 0, 0,
        ];
        assert_eq!(results.len(), 2);

        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert_eq!(
            Packet::RCChannels(RcChannelsPacked(expected)),
            results[1].clone().ok().unwrap()
        );
    }

    #[test]
    fn test_raw_to_full_packet_conversion() {
        let link_stats_packet = LinkStatistics {
            uplink_rssi_1: 16,
            uplink_rssi_2: 19,
            uplink_link_quality: 99,
            uplink_snr: 51 as i8,
            active_antenna: 1,
            rf_mode: 2,
            uplink_tx_power: 3,
            downlink_rssi: 8,
            downlink_link_quality: 88,
            downlink_snr: 48 as i8,
        };

        // Serialize it into a buffer
        let mut buffer = [0u8; 64];
        let bytes_written = write_packet_to_buffer(
            &mut buffer,
            PacketAddress::FlightController,
            &link_stats_packet,
        )
        .unwrap();
        let raw_bytes = &buffer[..bytes_written];

        let mut parser = CrsfParser::new();

        // 1. Parse raw bytes to get a RawCrsfPacket
        let mut raw_packet_result = Ok(None);
        for &byte in raw_bytes {
            raw_packet_result = parser.push_byte_raw(byte);
            if let Ok(Some(_)) = &raw_packet_result {
                break;
            }
        }
        let raw_packet = raw_packet_result
            .expect("Failed to get raw packet result")
            .expect("Expected a complete raw packet, but got None");

        let packet = Packet::parse(&raw_packet).expect("Failed to parse raw packet into a Packet");

        // Verify the resulting packet
        assert!(matches!(packet, Packet::LinkStatistics(_)));
        if let Packet::LinkStatistics(stats) = packet {
            assert_eq!(stats, link_stats_packet)
        }
    }
}
