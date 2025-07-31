use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;
use core::mem::size_of;

// Sub-type for the Timing Correction packet
const TIMING_CORRECTION_SUB_TYPE: u8 = 0x10;
const TIMING_CORRECTION_PAYLOAD_SIZE: usize = size_of::<u32>() + size_of::<i32>();

/// Represents a Remote-related packet (frame type 0x3A).
///
/// This is a container for various sub-packets related to remote functionality,
/// identified by a sub-type.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Remote {
    pub dst_addr: u8,
    pub src_addr: u8,
    pub payload: RemotePayload,
}

/// Enum for the different payloads of a Remote packet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RemotePayload {
    TimingCorrection(TimingCorrection),
    // Future subtypes can be added here.
}

/// Represents a Timing Correction (CRSF Shot) sub-packet (sub-type 0x10).
///
/// This packet is used for timing synchronization between the transmitter and receiver.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimingCorrection {
    /// Update interval in 100ns units.
    pub update_interval: u32,
    /// Timing offset in 100ns units.
    /// Positive values mean the data came too early, negative means late.
    pub offset: i32,
}

impl CrsfPacket for Remote {
    const PACKET_TYPE: PacketType = PacketType::RadioId;
    // Minimum payload for an extended header with a sub-type and its data.
    // For TimingCorrection: 1 (dst) + 1 (src) + 1 (sub-type) + 8 (data) = 11 bytes
    const MIN_PAYLOAD_SIZE: usize = 2 + 1 + TIMING_CORRECTION_PAYLOAD_SIZE;

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        // The `parse_extended_payload` helper is not used here because `Remote`
        // is a container for multiple sub-types. We need to dispatch based on
        // the sub-type manually.
        if data.len() < 3 {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        let dst_addr = data[0];
        let src_addr = data[1];
        let sub_type = data[2];
        let sub_payload = &data[3..];

        let payload = match sub_type {
            TIMING_CORRECTION_SUB_TYPE => {
                if sub_payload.len() < TIMING_CORRECTION_PAYLOAD_SIZE {
                    return Err(CrsfParsingError::InvalidPayloadLength);
                }
                let timing_correction = TimingCorrection {
                    update_interval: u32::from_be_bytes(
                        sub_payload[0..size_of::<u32>()]
                            .try_into()
                            .expect("infallible due to length check"),
                    ),
                    offset: i32::from_be_bytes(
                        sub_payload[size_of::<u32>()..TIMING_CORRECTION_PAYLOAD_SIZE]
                            .try_into()
                            .expect("infallible due to length check"),
                    ),
                };
                RemotePayload::TimingCorrection(timing_correction)
            }
            _ => return Err(CrsfParsingError::InvalidPayload), // Unknown sub-type
        };

        Ok(Self {
            dst_addr,
            src_addr,
            payload,
        })
    }

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        match &self.payload {
            RemotePayload::TimingCorrection(p) => {
                const LEN: usize = 2 + 1 + TIMING_CORRECTION_PAYLOAD_SIZE;
                if buffer.len() < LEN {
                    return Err(CrsfParsingError::BufferOverflow);
                }
                buffer[0] = self.dst_addr;
                buffer[1] = self.src_addr;
                buffer[2] = TIMING_CORRECTION_SUB_TYPE;
                buffer[3..7].copy_from_slice(&p.update_interval.to_be_bytes());
                buffer[7..11].copy_from_slice(&p.offset.to_be_bytes());
                Ok(LEN)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_correction_from_bytes() {
        // Full payload for a 0x3A packet
        let data: [u8; 11] = [
            0xEA, // dst_addr
            0xEE, // src_addr
            TIMING_CORRECTION_SUB_TYPE,
            0x00,
            0x00,
            0xC3,
            0x50, // update_interval = 50000
            0xFF,
            0xFF,
            0xFF,
            0xF9, // offset = -7
        ];
        let packet = Remote::from_bytes(&data).unwrap();
        assert_eq!(packet.dst_addr, 0xEA);
        assert_eq!(packet.src_addr, 0xEE);
        match packet.payload {
            RemotePayload::TimingCorrection(tc) => {
                assert_eq!(tc.update_interval, 50000);
                assert_eq!(tc.offset, -7);
            }
        }
    }

    #[test]
    fn test_timing_correction_to_bytes() {
        let packet = Remote {
            dst_addr: 0xEA,
            src_addr: 0xEE,
            payload: RemotePayload::TimingCorrection(TimingCorrection {
                update_interval: 50000,
                offset: -7,
            }),
        };
        let mut buffer = [0u8; 11];
        let len = packet.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 11);
        let expected: [u8; 11] = [
            0xEA,
            0xEE,
            TIMING_CORRECTION_SUB_TYPE,
            0x00,
            0x00,
            0xC3,
            0x50,
            0xFF,
            0xFF,
            0xFF,
            0xF9,
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_remote_round_trip() {
        let packet = Remote {
            dst_addr: 0xC8,
            src_addr: 0xEC,
            payload: RemotePayload::TimingCorrection(TimingCorrection {
                update_interval: 12345,
                offset: -6789,
            }),
        };
        let mut buffer = [0u8; 11];
        packet.to_bytes(&mut buffer).unwrap();
        let round_trip = Remote::from_bytes(&buffer).unwrap();
        assert_eq!(packet, round_trip);
    }

    #[test]
    fn test_from_bytes_invalid_len() {
        let data: [u8; 2] = [0; 2];
        let result = Remote::from_bytes(&data);
        assert!(matches!(
            result,
            Err(CrsfParsingError::InvalidPayloadLength)
        ));
    }

    #[test]
    fn test_from_bytes_unknown_subtype() {
        let data: [u8; 11] = [0xEA, 0xEE, 0x11, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = Remote::from_bytes(&data);
        assert!(matches!(result, Err(CrsfParsingError::InvalidPayload)));
    }
}
