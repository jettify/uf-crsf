use crate::constants;
use crate::error::CrsfStreamError;
use crate::packets::Packet;
use crate::packets::PacketAddress;
use crc::Crc;
use num_enum::TryFromPrimitive;

// Lengths are returned separately from borrows so both storage paths can use
// the same validator without borrowing the parser across buffer updates.
enum FrameStatus {
    /// A frame with valid sync, length, and CRC; contains its total byte length.
    Complete(usize),
    /// Contains the number of additional bytes needed for the next validation stage.
    Incomplete(usize),
}

#[derive(Debug)]
pub struct CrsfParser {
    buffer: [u8; constants::CRSF_MAX_PACKET_SIZE],
    start: usize,
    end: usize,
}

const CRC8_DVB_S2: Crc<u8> = Crc::<u8>::new(&crc::CRC_8_DVB_S2);

impl CrsfParser {
    pub fn new() -> Self {
        Self {
            buffer: [0; constants::CRSF_MAX_PACKET_SIZE],
            start: 0,
            end: 0,
        }
    }

    /// Iterates over owned, decoded packets from an input slice.
    ///
    /// Uses [`Self::next_raw_packet`] to avoid buffering contiguous frames,
    /// then decodes each frame into an owned [`Packet`]. Exhausting the iterator
    /// preserves partial frames for later chunks. To stop early without losing
    /// unread bytes, keep the caller-owned slice cursor and resume it later.
    /// Dropping the iterator does not consume or buffer any additional input.
    /// Errors are yielded as items; continue iterating to resume parsing.
    ///
    /// ```
    /// use uf_crsf::CrsfParser;
    ///
    /// let mut parser = CrsfParser::new();
    /// let bytes = [0xC8, 2, 0, 0, 0xC8, 2, 0, 0];
    /// let mut input = bytes.as_slice();
    /// let _ = parser.iter_packets(&mut input).next();
    /// assert_eq!(input, &bytes[4..]);
    ///
    /// // Single-byte sources use the same API. Drain errors as well as packets.
    /// parser.reset();
    /// for byte in &bytes {
    ///     let mut input = core::slice::from_ref(byte);
    ///     for result in parser.iter_packets(&mut input) {
    ///         let _ = result;
    ///     }
    ///     assert!(input.is_empty());
    /// }
    /// ```
    pub fn iter_packets<'p, 'c, 'i>(
        &'p mut self,
        input: &'c mut &'i [u8],
    ) -> PacketIterator<'p, 'c, 'i> {
        PacketIterator {
            parser: self,
            input,
        }
    }

    /// Advances `input` to the next decoded [`Packet`] or stream/parsing error.
    ///
    /// Contiguous frames are decoded directly from `*input` without copying into
    /// the parser's internal buffer; frames split across chunks use the internal
    /// buffer. Returns `None` when more input is needed, preserving any trailing
    /// partial frame and recovery bytes.
    pub fn next_packet(&mut self, input: &mut &[u8]) -> Option<Result<Packet, CrsfStreamError>> {
        self.next_raw_packet(input).map(|result| {
            result.and_then(|raw| Packet::parse(&raw).map_err(CrsfStreamError::ParsingError))
        })
    }

    /// Advances `input` to the next validated [`RawCrsfPacket`] or framing error.
    ///
    /// Contiguous frames borrow directly from `*input`; frames split across chunks
    /// borrow from the parser's internal buffer. Returns `None` when more input
    /// is needed, preserving any trailing partial frame for subsequent calls.
    ///
    /// A framing error discards only the rejected start byte. Later calls retry
    /// the remaining bytes, which can produce additional framing errors. Bytes
    /// retained during recovery are processed before new input, so a call can
    /// return a packet or error even when `input` is empty. Drain results until
    /// `None` before waiting for more data. A plausible length waits for enough
    /// bytes to validate its CRC before considering later frame starts.
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
        loop {
            // Internal buffer is empty (no incomplete bytes from prevous reads),
            // parser can continude with zero copy mode
            if self.start == self.end {
                self.reset();
                if input.is_empty() {
                    return None;
                }
                let chunk = *input;
                match Self::validate_candidate(chunk) {
                    Ok(FrameStatus::Complete(frame_len)) => {
                        *input = &chunk[frame_len..];
                        return Some(Ok(RawCrsfPacket {
                            bytes: &chunk[..frame_len],
                        }));
                    }
                    Err(error) => {
                        // Only this start has been disproved. Later bytes may
                        // contain a valid frame, including inside its payload.
                        *input = &chunk[1..];
                        return Some(Err(error));
                    }
                    Ok(FrameStatus::Incomplete(_)) => {
                        self.buffer[..chunk.len()].copy_from_slice(chunk);
                        self.end = chunk.len();
                        *input = &[];
                        return None;
                    }
                }
            }
            // Parser has incomplete data from prevous calls that needs
            // to be finished before moving into zero copy mode again
            match Self::validate_candidate(&self.buffer[self.start..self.end]) {
                Ok(FrameStatus::Complete(frame_len)) => {
                    let start = self.start;
                    self.start += frame_len;
                    return Some(Ok(RawCrsfPacket {
                        bytes: &self.buffer[start..self.start],
                    }));
                }
                Err(error) => {
                    self.start += 1;
                    return Some(Err(error));
                }
                Ok(FrameStatus::Incomplete(missing)) => {
                    if input.is_empty() {
                        return None;
                    }
                    let retained = self.end - self.start;
                    let count = missing.min(input.len());
                    if self.end + count > self.buffer.len() {
                        self.buffer.copy_within(self.start..self.end, 0);
                        self.start = 0;
                        self.end = retained;
                    }
                    self.buffer[self.end..self.end + count].copy_from_slice(&input[..count]);
                    self.end += count;
                    *input = &input[count..];
                }
            }
        }
    }

    /// Discards all partial frames and retained recovery bytes.
    pub fn reset(&mut self) {
        self.start = 0;
        self.end = 0;
    }

    fn validate_candidate(bytes: &[u8]) -> Result<FrameStatus, CrsfStreamError> {
        let Some(&sync) = bytes.first() else {
            return Ok(FrameStatus::Incomplete(1));
        };
        PacketAddress::try_from_primitive(sync).map_err(|_| CrsfStreamError::InvalidSync(sync))?;

        let Some(&length) = bytes.get(1) else {
            return Ok(FrameStatus::Incomplete(1));
        };
        let frame_len = usize::from(length) + 2;
        if !(constants::CRSF_MIN_PACKET_SIZE..=constants::CRSF_MAX_PACKET_SIZE).contains(&frame_len)
        {
            return Err(CrsfStreamError::InvalidPacketLength(length));
        }

        if bytes.len() < frame_len {
            return Ok(FrameStatus::Incomplete(frame_len - bytes.len()));
        }

        let calculated_crc = CRC8_DVB_S2.checksum(&bytes[2..frame_len - 1]);
        let packet_crc = bytes[frame_len - 1];
        if calculated_crc != packet_crc {
            return Err(CrsfStreamError::InvalidCrc {
                calculated_crc,
                packet_crc,
            });
        }
        Ok(FrameStatus::Complete(frame_len))
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

pub struct PacketIterator<'p, 'c, 'i> {
    parser: &'p mut CrsfParser,
    input: &'c mut &'i [u8],
}

impl<'i> PacketIterator<'_, '_, 'i> {
    /// Returns the unread input suffix with the original input lifetime.
    /// Recovery bytes already held by the parser are not part of this suffix.
    pub fn remaining(&self) -> &'i [u8] {
        self.input
    }
}

impl Iterator for PacketIterator<'_, '_, '_> {
    type Item = Result<Packet, CrsfStreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next_packet(self.input)
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
        let packet = parser
            .iter_packets(&mut &input[..])
            .next()
            .unwrap()
            .unwrap();
        assert!(matches!(packet, Packet::LinkStatistics(_)));
        assert_eq!(parser.buffer, [0xA5; constants::CRSF_MAX_PACKET_SIZE]);
        assert_eq!(parser.start, parser.end);
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
            assert_eq!(parser.start, parser.end);
        }
    }

    #[test]
    fn test_construction() {
        let raw_bytes: [u8; 14] = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
        let mut parser = CrsfParser::new();

        for b in &raw_bytes[0..raw_bytes.len() - 1] {
            let result = parser
                .next_raw_packet(&mut core::slice::from_ref(b))
                .transpose();
            assert!(matches!(result, Ok(None)));
        }

        let raw_packet_result = parser.next_raw_packet(&mut &raw_bytes[13..]).transpose();
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
            parser.iter_packets(&mut &raw_bytes[..]).collect();

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
            uplink_snr: 51,
            active_antenna: 1,
            rf_mode: 2,
            uplink_tx_power: 3,
            downlink_rssi: 8,
            downlink_link_quality: 88,
            downlink_snr: 48,
        };

        // Serialize it into a buffer
        let mut buffer = [0u8; 64];
        let bytes_written = write_packet_to_buffer(
            &mut buffer,
            PacketAddress::FlightController,
            &link_stats_packet,
        )
        .unwrap();
        let mut raw_bytes = &buffer[..bytes_written];

        let mut parser = CrsfParser::new();

        // 1. Parse raw bytes to get a RawCrsfPacket
        let raw_packet = parser.next_raw_packet(&mut raw_bytes).unwrap().unwrap();

        let packet = Packet::parse(&raw_packet).expect("Failed to parse raw packet into a Packet");

        // Verify the resulting packet
        assert!(matches!(packet, Packet::LinkStatistics(_)));
        if let Packet::LinkStatistics(stats) = packet {
            assert_eq!(stats, link_stats_packet)
        }
    }
}
